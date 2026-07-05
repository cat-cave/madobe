# shellcheck shell=bash
# shellcheck disable=SC1091,SC2016
# shellcheck source=scripts/qd-report-check/common.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"

validate_audit_status_values() {
  local file=$1

  expect_jq \
    "$file" \
    '(.acceptanceReviewed | type != "array" or all(.[]; type != "object" or (.status | type != "string") or (.status == "passed" or .status == "failed" or .status == "not_required")))' \
    "acceptanceReviewed.status must be passed, failed, or not_required"
}

validate_audit_findings() {
  local file=$1

  expect_jq "$file" '.findings | type == "array"' "findings must be an array"
  expect_jq "$file" '.findings | all(.[]; type == "object" and (.severity | nonblank) and (.title | nonblank) and (.evidence | nonblank) and (.observed | nonblank) and (.expected | nonblank) and (.classification | nonblank))' "findings must contain only objects with non-blank severity, title, evidence, observed, expected, and classification"
  expect_jq "$file" '.findings | all(.[]; type == "object" and (.severity as $severity | ["P0", "P1", "P2", "P3"] | index($severity)))' "findings[].severity must be one of P0, P1, P2, P3"
  expect_jq "$file" '.findings | all(.[]; type == "object" and (.classification as $classification | ["implementation", "spec-gap", "research-gap", "environment", "credential", "provider", "data", "policy", "regression"] | index($classification)))' "findings[].classification must be one of implementation, spec-gap, research-gap, environment, credential, provider, data, policy, regression"
  expect_jq "$file" '.findings | all(.[]; type == "object" and (if (has("path") and .path != null) then (.path | type == "string") and ((.path | gsub("^[[:space:]]+|[[:space:]]+$"; "") | length) == 0 or ((.path | startswith("/") | not) and (.path | split("/") | all(. != "..")))) else true end))' "findings[].path must be null, blank, or a repo-relative path without '..' traversal"
  expect_jq "$file" '.findings | all(.[]; type == "object" and (if (has("line") and .line != null) then (.line | type == "number" and . > 0 and . == floor) else true end))' "findings[].line must be null or a positive integer"
}

validate_audit() {
  local file=$1

  json_ok "$file" || return
  validate_node_id "$file"
  validate_real_world_status "$file"
  validate_audit_status_values "$file"

  expect_jq "$file" 'type == "object"' "top level must be an object"
  expect_jq "$file" '.nodeId | nonblank' "nodeId must be a non-blank string"
  expect_jq "$file" '.acceptanceReviewed | type == "array" and length > 0 and all(.[]; type == "object" and (.criterion | nonblank) and (.status | nonblank) and (.evidence | nonblank))' "acceptanceReviewed must be a non-empty array of review objects with non-blank criterion, status, and evidence"
  expect_jq "$file" '.verificationEvidence | type == "object"' "verificationEvidence must be an object"
  expect_jq "$file" '.verificationEvidence | (.diffReviewed == true) and (.completionReportReviewed == true) and (.verificationEvidenceReviewed == true)' "verificationEvidence diffReviewed, completionReportReviewed, and verificationEvidenceReviewed must all be true"
  expect_jq "$file" '.realWorldValidation | type == "object" and (.required | type == "boolean") and (.evidence | nonblank)' "realWorldValidation must include required boolean and non-blank evidence string"
  validate_audit_findings "$file"
}
