#include <gio/gio.h>
#include <gio/gunixfdlist.h>
#include <glib.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define DEST "org.freedesktop.portal.Desktop"
#define DESKTOP_PATH "/org/freedesktop/portal/desktop"
#define SCREENCAST_IFACE "org.freedesktop.portal.ScreenCast"
#define REQUEST_IFACE "org.freedesktop.portal.Request"

typedef struct {
  GDBusConnection *connection;
  GMainLoop *loop;
  const char *stage;
  const char *expected_path;
  guint response_code;
  GVariant *response_results;
  gboolean got_response;
  gboolean timed_out;
} Probe;

static gboolean on_timeout(gpointer user_data) {
  Probe *probe = user_data;
  probe->timed_out = TRUE;
  g_main_loop_quit(probe->loop);
  return G_SOURCE_REMOVE;
}

static void on_response(GDBusConnection *connection, const gchar *sender_name,
                        const gchar *object_path, const gchar *interface_name,
                        const gchar *signal_name, GVariant *parameters,
                        gpointer user_data) {
  (void)connection;
  (void)sender_name;
  (void)interface_name;
  (void)signal_name;

  Probe *probe = user_data;
  if (probe->expected_path == NULL ||
      g_strcmp0(object_path, probe->expected_path) != 0) {
    return;
  }

  guint response = 0;
  GVariant *results = NULL;
  g_variant_get(parameters, "(u@a{sv})", &response, &results);

  g_clear_pointer(&probe->response_results, g_variant_unref);
  probe->response_code = response;
  probe->response_results = results;
  probe->got_response = TRUE;

  g_print("response.stage=%s\n", probe->stage);
  g_print("response.path=%s\n", object_path);
  g_print("response.code=%u\n", response);
  gchar *printed = g_variant_print(results, TRUE);
  g_print("response.results=%s\n", printed);
  g_free(printed);

  g_main_loop_quit(probe->loop);
}

static GVariant *options_with_tokens(const char *handle_token,
                                     const char *session_token) {
  GVariantBuilder builder;
  g_variant_builder_init(&builder, G_VARIANT_TYPE("a{sv}"));
  g_variant_builder_add(&builder, "{sv}", "handle_token",
                        g_variant_new_string(handle_token));
  if (session_token != NULL) {
    g_variant_builder_add(&builder, "{sv}", "session_handle_token",
                          g_variant_new_string(session_token));
  }
  return g_variant_builder_end(&builder);
}

static GVariant *select_options(const char *handle_token) {
  GVariantBuilder builder;
  g_variant_builder_init(&builder, G_VARIANT_TYPE("a{sv}"));
  g_variant_builder_add(&builder, "{sv}", "handle_token",
                        g_variant_new_string(handle_token));
  g_variant_builder_add(&builder, "{sv}", "types", g_variant_new_uint32(1));
  g_variant_builder_add(&builder, "{sv}", "cursor_mode",
                        g_variant_new_uint32(1));
  g_variant_builder_add(&builder, "{sv}", "multiple", g_variant_new_boolean(FALSE));
  return g_variant_builder_end(&builder);
}

static GVariant *start_options(const char *handle_token) {
  GVariantBuilder builder;
  g_variant_builder_init(&builder, G_VARIANT_TYPE("a{sv}"));
  g_variant_builder_add(&builder, "{sv}", "handle_token",
                        g_variant_new_string(handle_token));
  return g_variant_builder_end(&builder);
}

static gboolean wait_for_response(Probe *probe, const char *stage,
                                  const char *request_path,
                                  guint timeout_seconds) {
  probe->stage = stage;
  probe->expected_path = request_path;
  probe->got_response = FALSE;
  probe->timed_out = FALSE;
  g_clear_pointer(&probe->response_results, g_variant_unref);

  guint timer = g_timeout_add_seconds(timeout_seconds, on_timeout, probe);
  g_main_loop_run(probe->loop);
  g_source_remove(timer);

  if (probe->timed_out) {
    g_print("response.stage=%s\n", stage);
    g_print("response.path=%s\n", request_path);
    g_print("response.timeout_seconds=%u\n", timeout_seconds);
    return FALSE;
  }
  return probe->got_response;
}

static char *call_request(Probe *probe, const char *method, GVariant *parameters,
                          GError **error) {
  GVariant *reply = g_dbus_connection_call_sync(
      probe->connection, DEST, DESKTOP_PATH, SCREENCAST_IFACE, method, parameters,
      G_VARIANT_TYPE("(o)"), G_DBUS_CALL_FLAGS_NONE, -1, NULL, error);
  if (reply == NULL) {
    return NULL;
  }

  const char *request_path = NULL;
  g_variant_get(reply, "(&o)", &request_path);
  char *copy = g_strdup(request_path);
  g_variant_unref(reply);
  g_print("request.method=%s\n", method);
  g_print("request.path=%s\n", copy);
  return copy;
}

static char *require_session_handle(Probe *probe) {
  if (probe->response_results == NULL) {
    return NULL;
  }

  const char *lookup_path = NULL;
  if (g_variant_lookup(probe->response_results, "session_handle", "&o",
                       &lookup_path) ||
      g_variant_lookup(probe->response_results, "session_handle", "&s",
                       &lookup_path)) {
    g_print("session.handle=%s\n", lookup_path);
    return g_strdup(lookup_path);
  }

  GVariant *value = g_variant_lookup_value(probe->response_results,
                                           "session_handle", NULL);
  if (value == NULL) {
    return NULL;
  }

  GVariant *unboxed = value;
  if (g_variant_is_of_type(value, G_VARIANT_TYPE_VARIANT)) {
    unboxed = g_variant_get_variant(value);
  }
  if (!g_variant_is_of_type(unboxed, G_VARIANT_TYPE_OBJECT_PATH)) {
    if (unboxed != value) {
      g_variant_unref(unboxed);
    }
    g_variant_unref(value);
    return NULL;
  }

  char *session_handle = g_strdup(g_variant_get_string(unboxed, NULL));
  if (unboxed != value) {
    g_variant_unref(unboxed);
  }
  g_variant_unref(value);
  g_print("session.handle=%s\n", session_handle);
  return session_handle;
}

static void print_streams(Probe *probe) {
  GVariant *streams = NULL;
  if (probe->response_results == NULL ||
      !g_variant_lookup(probe->response_results, "streams", "@a(ua{sv})",
                        &streams)) {
    g_print("streams.present=false\n");
    return;
  }

  g_print("streams.present=true\n");
  gsize count = g_variant_n_children(streams);
  g_print("streams.count=%zu\n", count);
  for (gsize i = 0; i < count; i++) {
    guint32 node_id = 0;
    GVariant *properties = NULL;
    g_variant_get_child(streams, i, "(u@a{sv})", &node_id, &properties);
    g_print("stream.%zu.node_id=%u\n", i, node_id);
    gchar *printed = g_variant_print(properties, TRUE);
    g_print("stream.%zu.properties=%s\n", i, printed);
    g_free(printed);
    g_variant_unref(properties);
  }
  g_variant_unref(streams);
}

static int open_pipewire_remote(Probe *probe, const char *session_handle,
                                GError **error) {
  GVariantBuilder builder;
  g_variant_builder_init(&builder, G_VARIANT_TYPE("a{sv}"));
  GUnixFDList *out_fd_list = NULL;
  GVariant *reply = g_dbus_connection_call_with_unix_fd_list_sync(
      probe->connection, DEST, DESKTOP_PATH, SCREENCAST_IFACE,
      "OpenPipeWireRemote", g_variant_new("(oa{sv})", session_handle, &builder),
      G_VARIANT_TYPE("(h)"), G_DBUS_CALL_FLAGS_NONE, -1, NULL, &out_fd_list, NULL,
      error);
  if (reply == NULL) {
    g_clear_object(&out_fd_list);
    return -1;
  }

  gint fd_index = -1;
  g_variant_get(reply, "(h)", &fd_index);
  g_variant_unref(reply);

  int fd = g_unix_fd_list_get(out_fd_list, fd_index, error);
  g_clear_object(&out_fd_list);
  if (fd >= 0) {
    g_print("open_pipewire_remote.called=true\n");
    g_print("open_pipewire_remote.fd_received=true\n");
  }
  return fd;
}

static guint parse_uint_arg(int argc, char **argv, const char *name,
                            guint fallback) {
  for (int i = 1; i + 1 < argc; i++) {
    if (strcmp(argv[i], name) == 0) {
      return (guint)strtoul(argv[i + 1], NULL, 10);
    }
  }
  return fallback;
}

int main(int argc, char **argv) {
  guint response_timeout = parse_uint_arg(argc, argv, "--response-timeout", 90);
  guint hold_seconds = parse_uint_arg(argc, argv, "--hold-seconds", 30);

  GError *error = NULL;
  Probe probe = {0};
  probe.connection = g_bus_get_sync(G_BUS_TYPE_SESSION, NULL, &error);
  if (probe.connection == NULL) {
    g_printerr("error.stage=connect-session-bus\nerror.message=%s\n",
               error->message);
    g_clear_error(&error);
    return 2;
  }
  probe.loop = g_main_loop_new(NULL, FALSE);

  guint subscription = g_dbus_connection_signal_subscribe(
      probe.connection, DEST, REQUEST_IFACE, "Response", NULL, NULL,
      G_DBUS_SIGNAL_FLAGS_NONE, on_response, &probe, NULL);

  char *create_request = call_request(
      &probe, "CreateSession",
      g_variant_new("(@a{sv})",
                    options_with_tokens(
                        "madobe_m2_portal_manual_consent_frame_proof_create",
                        "madobe_m2_portal_manual_consent_frame_proof_session")),
      &error);
  if (create_request == NULL) {
    g_printerr("error.stage=CreateSession\nerror.message=%s\n", error->message);
    g_clear_error(&error);
    return 3;
  }
  if (!wait_for_response(&probe, "CreateSession", create_request,
                         response_timeout) ||
      probe.response_code != 0) {
    g_free(create_request);
    return 4;
  }

  char *session_handle = require_session_handle(&probe);
  if (session_handle == NULL) {
    g_printerr("error.stage=CreateSession\nerror.message=missing-session-handle\n");
    g_free(create_request);
    return 5;
  }

  char *select_request =
      call_request(&probe, "SelectSources",
                   g_variant_new("(o@a{sv})", session_handle,
                                 select_options(
                                     "madobe_m2_portal_manual_consent_frame_proof_select")),
                   &error);
  if (select_request == NULL) {
    g_printerr("error.stage=SelectSources\nerror.message=%s\n", error->message);
    g_clear_error(&error);
    g_free(session_handle);
    g_free(create_request);
    return 6;
  }
  if (!wait_for_response(&probe, "SelectSources", select_request,
                         response_timeout) ||
      probe.response_code != 0) {
    g_free(select_request);
    g_free(session_handle);
    g_free(create_request);
    return 7;
  }

  g_print("manual.action=If a chooser appears, select only the node-scoped madobe output and approve sharing.\n");
  char *start_request =
      call_request(&probe, "Start",
                   g_variant_new("(os@a{sv})", session_handle, "",
                                 start_options(
                                     "madobe_m2_portal_manual_consent_frame_proof_start")),
                   &error);
  if (start_request == NULL) {
    g_printerr("error.stage=Start\nerror.message=%s\n", error->message);
    g_clear_error(&error);
    g_free(select_request);
    g_free(session_handle);
    g_free(create_request);
    return 8;
  }
  if (!wait_for_response(&probe, "Start", start_request, response_timeout) ||
      probe.response_code != 0) {
    g_free(start_request);
    g_free(select_request);
    g_free(session_handle);
    g_free(create_request);
    return 9;
  }
  print_streams(&probe);

  int pipewire_fd = open_pipewire_remote(&probe, session_handle, &error);
  if (pipewire_fd < 0) {
    g_printerr("error.stage=OpenPipeWireRemote\nerror.message=%s\n",
               error->message);
    g_clear_error(&error);
    g_free(start_request);
    g_free(select_request);
    g_free(session_handle);
    g_free(create_request);
    return 10;
  }

  g_print("hold.seconds=%u\n", hold_seconds);
  sleep(hold_seconds);
  close(pipewire_fd);

  g_free(start_request);
  g_free(select_request);
  g_free(session_handle);
  g_free(create_request);
  g_clear_pointer(&probe.response_results, g_variant_unref);
  g_dbus_connection_signal_unsubscribe(probe.connection, subscription);
  g_main_loop_unref(probe.loop);
  g_object_unref(probe.connection);
  return 0;
}
