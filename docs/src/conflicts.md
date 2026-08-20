# Conflicts

Today the app has one kind of conflict — a **file name** clash on copy, move,
or import from Finder.

The dialog offers Replace, Keep Both, or Skip. A checkbox applies the choice
to the rest of the clashes in the same operation. Replace snapshots the
content that would be overwritten first.

**Version conflict** (`base_hash` diverged while the document was open):
`write_doc` returns an error and **does not write**. A banner to keep your
version, reload from disk, or show a diff will arrive with the editor UI.
