#!/usr/bin/env python3
"""Extract SQL examples from the PostgreSQL documentation sources.

The PostgreSQL docs put runnable SQL in <programlisting> blocks, usually
followed by the psql output it produces. This keeps the statement and drops
the output, writing one record per statement to a corpus file that
tests/token_loss_test.rs reads.

    python3 scripts/extract_doc_corpus.py \
        ~/Source/gmr/postgres/doc/src/sgml \
        tests/fixtures/corpus/postgres_doc.sql

Regenerate only when moving to a new PostgreSQL major version; the corpus is
committed so the test needs no PostgreSQL checkout.
"""

import collections
import glob
import os
import re
import sys

# A block is SQL when its first word is one of these.
SQL_START = set(
    """select insert update delete merge with table values
    create alter drop truncate comment grant revoke
    begin commit rollback savepoint release start
    copy explain analyze vacuum reindex cluster refresh
    set reset show declare fetch move close prepare execute deallocate
    listen notify unlisten do call checkpoint load lock discard
    import reassign security""".split()
)

ENTITIES = {
    "&lt;": "<",
    "&gt;": ">",
    "&amp;": "&",
    "&quot;": '"',
    "&ldquo;": '"',
    "&rdquo;": '"',
}


def first_statement(text):
    """The first complete statement in `text`, or None.

    Cuts at the first semicolon that ends a line outside a quoted string,
    which drops the psql output the docs print after the statement.
    """
    quote = None  # "'", '"', or a dollar-quote tag
    i = 0
    while i < len(text):
        c = text[i]
        if quote is None:
            if c in "'\"":
                quote = c
            elif c == "$":
                m = re.match(r"\$[A-Za-z_]*\$", text[i:])
                if m:
                    quote = m.group(0)
                    i += len(quote)
                    continue
            elif c == ";":
                rest = text[i + 1 :]
                if not rest.strip() or rest.lstrip(" \t").startswith("\n"):
                    return text[: i + 1]
        elif quote in "'\"":
            if c == quote:
                quote = None
        elif text.startswith(quote, i):
            i += len(quote)
            quote = None
            continue
        i += 1
    return None


def main():
    sgml, out_path = sys.argv[1], sys.argv[2]
    stats = collections.Counter()
    records = []

    for path in sorted(glob.glob(os.path.join(sgml, "**", "*.sgml"), recursive=True)):
        src = open(path, encoding="utf-8", errors="replace").read()
        for m in re.finditer(r"<programlisting[^>]*>(.*?)</programlisting>", src, re.S):
            raw = m.group(1)
            stats["blocks"] += 1
            if "<replaceable" in raw:
                stats["skip: placeholder"] += 1
                continue
            text = re.sub(r"<[^>]+>", "", raw)
            for entity, char in ENTITIES.items():
                text = text.replace(entity, char)
            text = text.strip()
            if not text:
                stats["skip: empty"] += 1
                continue
            words = text.split()
            first = re.sub(r"[^a-z]", "", words[0].lower()) if words else ""
            if first not in SQL_START:
                stats["skip: not sql"] += 1
                continue
            if re.search(r"^\s*\\", text, re.M) or "=>" in text or "=#" in text:
                stats["skip: psql session"] += 1
                continue
            if "..." in text:
                # An elision the docs write in running SQL, not a placeholder
                # <replaceable>, so it reads as an example but does not run.
                stats["skip: elision"] += 1
                continue
            stmt = first_statement(text)
            if stmt is None:
                stats["skip: no terminator"] += 1
                continue
            stats["kept"] += 1
            records.append((os.path.relpath(path, sgml), stmt))

    with open(out_path, "w", encoding="utf-8") as f:
        f.write(
            "-- SQL examples extracted from the PostgreSQL documentation by\n"
            "-- scripts/extract_doc_corpus.py. Records are separated by a line\n"
            "-- holding `;;`. Each record is preceded by its source file.\n"
            "--\n"
            "-- Copyright (c) 1996-2025, PostgreSQL Global Development Group.\n"
            "-- Used under the PostgreSQL License.\n"
        )
        for source, stmt in records:
            f.write(f";;\n-- {source}\n{stmt}\n")

    for key, count in stats.most_common():
        print(f"{key:22} {count}")


if __name__ == "__main__":
    main()
