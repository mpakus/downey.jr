# Privacy

Documents do not leave the computer. There is no account, no analytics, and
no CDN for themes, fonts, Mermaid, or KaTeX.

The only outgoing request is an update check from the button or
File → Check for Updates…. It talks to GitHub Releases and does not include
documents. Signed in-app install is still ahead and will stay optional.

`asset://` serves files only from registered project roots. `javascript:`,
`data:`, and local `file:` links in a document are not used as navigation.

Logs record actions and errors, not Markdown contents.
