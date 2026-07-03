import MadobeClientCore
import XCTest

final class MadobeClientCoreTests: XCTestCase {
    func testIdentityMatchesRustBootstrapProof() {
        XCTAssertEqual(
            MadobeClientCore.identity().statusLine,
            "madobe 0.1.0 protocol=1 event=madobe.bootstrap ts=0 status=ok"
        )
    }
}
