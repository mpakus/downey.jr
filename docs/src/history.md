# Version history

The history pane and `pre_*` snapshots arrive with the editor (P10). The
reader does not rewrite documents on its own, so there is nothing to roll
back yet.

Destructive tree operations (replace on copy, Trash) already take internal
snapshots before they change files. There is no user UI to restore those
snapshots yet.
