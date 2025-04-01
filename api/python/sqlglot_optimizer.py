#!/usr/bin/env python3
import sqlglot
import sys
from sqlglot.optimizer import optimize
from sqlglot.optimizer.qualify_columns import quote_identifiers

def main():

    RULES = (
    quote_identifiers,
)

    sql = sys.argv[1]
    optimized_sql = optimize(sql, rules=RULES).sql(pretty=True)
    postgres_sql=sqlglot.transpile(optimized_sql, write="postgres")[0]
    print(postgres_sql)

if __name__ == "__main__":
    main()
