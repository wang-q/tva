# Field Syntax

All tools use a unified syntax to identify fields (columns). This syntax allows selecting fields by index, name, range, or wildcard.

*   1-based Indexing
  - Fields are numbered starting from 1 (following Unix `cut`/`awk` convention).
  - Example: `1,3,5` selects the 1st, 3rd, and 5th columns.

* Field Names
  - Requires the `--header` flag (or command-specific header option).
  - Names are case-sensitive.
  - Example: `date,user_id` selects columns named "date" and "user_id".

* Ranges
  - Numeric Ranges: `start-end`. Example: `2-4` selects columns 2, 3, and 4.
  - Name Ranges: `start_col-end_col`. Selects all columns from `start_col` to `end_col` inclusive, based on their order in the header.
  - Reverse Ranges: `5-3` is automatically treated as `3-5`.

* Wildcards
  - `*` matches any sequence of characters in a field name.
  - Example: `user_*` selects `user_id`, `user_name`, etc.
  - Example: `*_time` selects `start_time`, `end_time`.

* Escaping
  - Special characters in field names (like space, comma, colon, dash, star) must be escaped with `\`.
  - Example: `Order\ ID` selects the column "Order ID".
  - Example: `run\:id` selects "run:id".

* Exclusion
  - Negative selection is typically handled via a separate flag (e.g., `--exclude` in `select`), but uses the same field syntax.
