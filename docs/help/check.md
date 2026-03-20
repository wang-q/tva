# check

Validates the structure of TSV input by ensuring that all lines have the
same number of fields.

Behavior:

* Without header mode: The number of fields on the first line is used as the expected count.
* With header mode: The number of fields in the header's column names line is used as the expected
  count.
* Each subsequent line must have the same number of fields.
* On mismatch, details about the failing line and expected field count are
  printed to stderr and the command exits with a non-zero status.

Input:

* Reads from files or standard input; multiple files are processed as one stream.
* Files ending in `.gz` are transparently decompressed.

Header behavior:

* Supports all four header modes. See `tva --help-headers` for details.
* When header mode is enabled, the header lines are skipped from structure checking.
* The field count from the header's column names line is used as the expected count.

Output:

* On success, prints: `<N> lines total, <M> data lines, <P> fields`