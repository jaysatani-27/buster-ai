#!/usr/bin/env python3
import sqlglot
import sys

def main():
    sql = sys.argv[1]
    dialect = sys.argv[2]
    transpiled_sql = sqlglot.transpile(sql, read="postgres", write=dialect)[0]
    print(transpiled_sql)

if __name__ == "__main__":
    main()
