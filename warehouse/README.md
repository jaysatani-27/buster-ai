# Buster Warehouse Overview

In working with our customers, we found that Snowflake, Bigquery, and other warehouse solutions were prohibitively expensive or slow in them being able to deploy AI-powered analytics at scale.

Additionaly, we found that having a close integration between the data warehouse and our AI-native BI tool allows for a better and more reliable data experience.

### Key Features

- **Built on Starrocks:** We felt that Starrock was the best query engine by default for our use case. The main thing that pushed us towards it was that they perform predicate pushdown on iceberg tables, whereas Clickhouse and DuckDB do not.  We were also impressed by the performance, caching system, and flexibility of Starrocks.
- **Built on Apache Iceberg:** Some of the top companies in the world use Apache Iceberg for storing and interacting with their data.  We wanted a table format that not only brought tremendous benefits, but one that companies wouldn't outgrow.
- **Bring Your Own Storage:** We felt that customers should own their data and not be locked into a particular storage engine.

## Quickstart

1. Dependencies:
   - Make sure that you have [Docker Engine](https://docs.docker.com/engine/install/) installed.
   - Install [Python](https://www.python.org/downloads/) if you haven't already.
   - Install a [MySQL client](https://dev.mysql.com/downloads/mysql/) on your system.
   - An AWS account with S3 access.

2. Clone the repository:

```bash
git clone https://github.com/buster-so/warehouse.git
```

3. Run the warehouse:

```bash
docker compose up -d
```

4. Populate the `.env` file with AWS credentials provisioned for S3 access. **Note: You can use any S3 compatible storage, you might just need to tweak some of the configs.** Feel free to look at the Starrocks [docs](https://docs.starrocks.com/en-us/main/loading/iceberg/iceberg_external_catalog) or PyIceberg [docs](https://iceberg.apache.org/docs/latest/spark-configuration/) for more information.

5. Connect to the warehouse with any MySQL client.

6. Create the external catalog:

```sql
CREATE EXTERNAL CATALOG 'public'
PROPERTIES
(
  "type"="iceberg",
  "iceberg.catalog.type"="rest",
  "iceberg.catalog.uri"="http://iceberg-rest:8181",
  "iceberg.catalog.warehouse"="<BUCKET_NAME>",
  "aws.s3.access_key"="<ACCESS_KEY>",
  "aws.s3.secret_key"="<SECRET_KEY>",
  "aws.s3.region" = "<REGION>",
  "aws.s3.enable_path_style_access"="true",
  "client.factory"="com.starrocks.connector.iceberg.IcebergAwsClientFactory"
);
```

7. Seed the data. If you want to populate a table with 75m records, you can run the notebook found [here](/notebooks/populate_warehouse.ipynb).

8. Set the catalog

```sql
SET CATALOG 'public';
```

9. Set the database

```sql
USE DATABASE 'public';
```

10. Run a query

```sql
SELECT COUNT(*) FROM public.nyc_taxi;
```

### Optimizations

For data that you think will be accessed frequently, you can cache it on disk for faster access with:

```sql
CACHE SELECT * FROM public.nyc_taxi WHERE tpep_pickup_datetime > '2022-03-01';
```

## Deployment on AWS

WIP


## Shoutouts

The documentation from the Starrocks, Iceberg, and PyIceberg team has been very helpful in building this project.
