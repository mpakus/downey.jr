# FAQ

**Why can’t I type in the preview?** This is a reader. A richer editor is the
next large phase. For now: ⌘⇧O, or Edit / Split with ⌘S.

**A project disappeared from the list?** The folder was probably moved.
Context menu → Find Folder….

**The diagram is empty.** Check Settings → Render Mermaid diagrams. If it is
on, check the fence syntax; errors show under the source.

**Formulas don’t render.** Settings → mathematics (KaTeX). You need `$...$` /
`$$...$$` in the Markdown, as in the renderer corpus.

**Release 0.3.0 won’t open from the DMG.** That release was not notarized. Use
Right-click → Open / `xattr -cr` as in [Install](install.md), or download a
newer signed release. From source, use `cargo tauri dev`.

**I closed the window and the menu-bar icon is still there?** That is
intentional. Click the icon to show the window; Quit in its menu or ⌘Q
quits the process.

**Is there a cloud, account, or telemetry?** No. Update checking runs only
when you choose File → Check for Updates… or the button in About; it asks
GitHub Releases and does not send documents.
