// Puts text on the macOS pasteboard marked as concealed, the way a password manager does,
// so that "clipbuf must not capture this" (requirement 1.8) can be checked without owning one.
//
//   swift scripts/macos-concealed-copy.swift "secret text"
//
// clipbuf must NOT add the text to its list. Anything else is a bug.
//
// `org.nspasteboard.ConcealedType` is the convention password managers follow on macOS; the
// two Windows and KDE markers clipbuf also honours have no meaning here and are not written.

import AppKit

let text = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "concealed test value"

let pasteboard = NSPasteboard.general
pasteboard.clearContents()
// The text itself, so the pasteboard looks ordinary to whatever pastes it...
pasteboard.setString(text, forType: .string)
// ...plus the marker that asks clipboard managers to leave it alone.
pasteboard.setString("", forType: NSPasteboard.PasteboardType("org.nspasteboard.ConcealedType"))

print("copied \(text.count) characters, marked org.nspasteboard.ConcealedType")
print("clipbuf must not list it")
