# join

Joins lines from a TSV data stream against a filter file using one or more key
fields.

Behavior:

* Reads the filter file into memory and builds a hash map of keys to append values.
* Processes data files sequentially, extracting keys and looking up matches.
* Supports inner join (default), left outer join (--write-all), and anti-join (--exclude).
* When using --header, field names can be used in key-fields, data-fields, and append-fields.
* Keys are compared as byte strings for exact matching.
* By default, duplicate keys in the filter file with different append values will cause an error.
  Use `--allow-duplicate-keys` / `-z` to allow duplicates (last entry wins).

Input:

* The filter file is specified with `--filter-file` / `-f` and is read into memory.
* Data is read from files or standard input.
* Files ending in `.gz` are transparently decompressed.

Output:

* By default, only matching lines from the data stream are written (inner join).
* Use `--write-all` / `-w` to output all data records with a fill value for unmatched rows (left outer join).
* Use `--exclude` / `-e` to output only non-matching records (anti-join).
* By default, output is written to standard output.
* Use `--outfile` / `-o` to write to a file instead.

Header behavior:

* Supports `--header` / `-H` and `--header-hash1` modes.
* When using header mode, exactly one header line is written at the top of output.
* Appended fields from the filter file are added to the data header with an optional prefix.

Keys:

* `--key-fields` / `-k`: Selects key fields from the filter file (default: 0 = entire line).
* `--data-fields` / `-d`: Selects key fields from the data stream, if different from --key-fields.
* Use 0 to indicate the entire line should be used as the key.
* Multiple fields can be specified for composite keys (e.g., "1,2" or "col1,col2").

Append fields:

* `--append-fields` / `-a`: Specifies fields from the filter file to append to matching records.
* Fields are appended in the order specified, separated by the delimiter.
* Use `--prefix` / `-p` to add a prefix to appended header field names.

Field syntax:

* Field lists support 1-based indices, ranges (`1-3,5-7`), header names, name
  ranges (`run-user_time`), and wildcards (`*_time`).
* Run `tva --help-fields` for a full description shared across tva commands.

Examples:

1. Basic inner join using entire line as key
   `tva join -f filter.tsv data.tsv`

2. Join on specific column by index
   `tva join -f filter.tsv -k 1 -d 2 data.tsv`

3. Join using header field names, appending specific columns
   `tva join -H -f filter.tsv -k id -a name,value data.tsv`

4. Left outer join (output all data rows with fill value for non-matches)
   `tva join -H -f filter.tsv -k id -a name --write-all "NA" data.tsv`

5. Anti-join (output only non-matching rows)
   `tva join -H -f filter.tsv -k id --exclude data.tsv`

6. Multi-key join with different key fields in filter and data
   `tva join -H -f filter.tsv -k first,last -d fname,lname data.tsv`

7. Use custom delimiter and append fields with prefix
   `tva join --delimiter ":" -H -f filter.tsv -k 1 -a 2,3 --prefix "f_" data.tsv`
