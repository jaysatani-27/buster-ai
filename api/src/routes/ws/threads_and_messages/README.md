# Threads

There are 4 main subscriptions for threads:

- [`/threads/list`](#threadslist): Listens for state changes to threads in the user's access.
- [`/threads/get`](#threadsget): Listens for state changes in a thread.
- [`/threads/post`](#threadspost): Listens for events while creating/chatting in a thread.
- [`/threads/update`](#threadsupdate): Listens for events while updating a thread.

There are 3 requests that don't subscribe to a channel and just return a state response or disconnect immediately:

- [`/threads/delete`](#threadsdelete): Deletes a thread.
- [`/threads/unsubscribe`](#threadsunsubscribe): Unsubscribes from a channel.

## `/threads/list`

When a user requests `/threads/list`, they are subscribed to their own channel that listens for any new threads that are added to their access.

### Request

```json
{
    "route": "/threads/list",
    "data": {
        "page_size": 25, //Nullable defaults to 25.
        "page_token": 0, //Nullable defaults to 0.
    }
}
```

## Events/Responses

*The response types look the same except for the event type.*

### `getThreadsList`

This is the initial response that is returned when a user requests `/threads/list`.

 ```json
 {
    "route": "/threads/list",
    "event": "getThreadsList",
    "payload": [
        {
            "id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "title": "A Cool Title",
            "last_edited": "2024-06-11T16:35:51.575118Z",
            "dataset_name": "A Cool Dataset",
            "dataset_uuid": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "created_by_id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "created_by_name": "Chad",
            "created_by_email": "chad@buster.so",
            "status": "verified",
        },
        ...
    ]
}
 ```

### `updateThreadsList`

this is sent when the threads state is updated.

```json
 {
    "route": "/threads/list",
    "event": "updateThreadsList",
    "payload": [
        {
            "id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "title": "A Cool Title",
            "last_edited": "2024-06-11T16:35:51.575118Z",
            "dataset_name": "A Cool Dataset",
            "dataset_uuid": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "created_by_id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "created_by_name": "Chad",
            "created_by_email": "chad@buster.so",
            "status": "verified", // 'verified', 'notVerified', 'rejected'
        },
        ...
    ]
}
```

### Unsubscribing

To unsubscribe, the user can send a request to `/threads/unsubscribe` that has a null `id` field.

#### Request

```json
{
    "route": "/threads/unsubscribe",
    "data": {
        "id": null
    }
}
```

#### Response

```json
{
    "route": "/threads/unsubscribe",
    "event": "unsubscribed",
    "payload": null
}
```

## `/threads/get`

When a user requests `/threads/get`, they are automatically subscribed to a channel for that thread.

### Request

```json
{
    "route": "/threads/get",
    "data": {
        "id": "3912181d-0f67-4b92-b340-37fa11dddc4c"
    }
}
```

## Events/Responses

*The response types for "presence" of when a user joins or leaves a chat are the same except for the event type.*

*The response types for "state" of a chat are the same except for the event type.*

### `joinedThread`

```json
{
    "route": {
        "Threads": "/threads/get"
    },
    "event": {
        "Threads": "joinedThread"
    },
    "payload": [
        {
            "email": "chad@buster.so",
            "id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "name": "Chad",
            "thread_id": "3912181d-0f67-4b92-b340-37fa11dddc4c"
        }
    ],
    "error": null
}
```

### `leftThread`

```json
{
    "route": {
        "Threads": "/threads/get"
    },
    "event": {
        "Threads": "leftThread"
    },
    "payload": [
        {
            "email": "chad@buster.so",
            "id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "name": "Chad", // Nullable Field
            "thread_id": "3912181d-0f67-4b92-b340-37fa11dddc4c"
        }
    ],
    "error": null
}
```

### `getThreadState`

this is the initial response that is returned when a user requests `/threads/get`.

```json
{
    "route": {
        "Threads": "/threads/get"
    },
    "event": {
        "Threads": "getThreadState"
    },
    "payload": [
        {
            "created_at": "2024-06-11T16:35:51.575118Z",
            "created_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "dashboard_id": "e59aef54-9f6f-40dd-8631-d85c8d3cbd9c",
            "deleted_at": null,
            "folder_id": null,
            "id": "3912181d-0f67-4b92-b340-37fa11dddc4c",
            "messages": [
                {
                    "chart_recommendations": null, // Nullable
                    "code": null, // Nullable
                    "column_metadata": null, // Nullable
                    "context": {
                      "steps": [
                        {
                          "identifyingDataset": {
                            "dataset": {
                              "id": "5381a3ac-9671-4318-a8d3-3dc34b7d3c42",
                              "name": "Sales Dataset"
                            },
                            "progress": "completed",
                            "threadId": "6c7668cc-b82f-4b38-9a6a-965049d4aa1c",
                            "messageId": "045b7723-d758-4c3d-9b3c-f912e8b0083a"
                          }
                        },
                        {
                          "generatingSql": {
                            "sql": "\nWITH monthly_sales AS (\n    SELECT \n        DATE_TRUNC('month', order_date) AS month,\n        SUM(total_sales_amount) AS total_sales\n    FROM \n        (SELECT DISTINCT order_id, order_date, total_sales_amount FROM sales_summary) AS deduped_sales\n    GROUP BY \n        DATE_TRUNC('month', order_date)\n)\nSELECT \n    TO_CHAR(month, 'YYYY-MM') AS month,\n    total_sales\nFROM \n    monthly_sales\nORDER BY \n    total_sales DESC\nLIMIT 1;",
                            "progress": "completed",
                            "sqlChunk": null,
                            "threadId": "6c7668cc-b82f-4b38-9a6a-965049d4aa1c",
                            "messageId": "045b7723-d758-4c3d-9b3c-f912e8b0083a"
                          }
                        },
                        {
                          "fetchingData": {
                            "data": null,
                            "progress": "completed",
                            "threadId": "6c7668cc-b82f-4b38-9a6a-965049d4aa1c",
                            "messageId": "045b7723-d758-4c3d-9b3c-f912e8b0083a"
                          }
                        }
                      ]
                    }, // Nullable
                    "created_at": "2024-06-11T16:35:51.576164Z",
                    "dataset_id": "5381a3ac-9671-4318-a8d3-3dc34b7d3c42", // Nullable
                    "deleted_at": null, // Nullable
                    "feedback": null, // Nullable
                    "id": "3b344d39-61e3-4f4e-ac36-c7239a836a0a",
                    "message": "this is great, can you break that down week by week",
                    "responses": { // nullable
                        "messages": [
                            "this is a response",
                            "this is another response",
                        ]
                    },
                    "sent_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
                    "sent_by_name": "Chad", 
                    "thread_id": "3912181d-0f67-4b92-b340-37fa11dddc4c",
                    "time_frame": null, // Nullable
                    "title": null, // Nullable
                    "updated_at": null // Nullable
                }
                ...
            ],
            "updated_at": null,
            "updated_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e"
        }
    ],
    "error": null
}
```

### `updateThreadState`

this is sent when the threads state is updated.

```json
{
    "route": {
        "Threads": "/threads/get"
    },
    "event": {
        "Threads": "updateThreadState"
    },
    "payload": [
        {
            "created_at": "2024-06-11T16:35:51.575118Z",
            "created_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "dashboard_id": "e59aef54-9f6f-40dd-8631-d85c8d3cbd9c", // Nullable
            "deleted_at": null, // Nullable
            "folder_id": null, // Nullable
            "id": "3912181d-0f67-4b92-b340-37fa11dddc4c",
            "messages": [
                {
                    "chart_recommendations": null, // Nullable
                    "code": null, // Nullable
                    "column_metadata": null, // Nullable
                    "context": null, // Nullable
                    "created_at": "2024-06-11T16:35:51.576164Z",
                    "dataset_id": "5381a3ac-9671-4318-a8d3-3dc34b7d3c42", // Nullable
                    "deleted_at": null, // Nullable
                    "feedback": null, // Nullable
                    "id": "3b344d39-61e3-4f4e-ac36-c7239a836a0a",
                    "message": "this is great, can you break that down week by week",
                    "responses": { // nullable
                        "messages": [
                            "this is a response",
                            "this is another response",
                        ]
                    },
                    "sent_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
                    "sent_by_name": "Chad", 
                    "thread_id": "3912181d-0f67-4b92-b340-37fa11dddc4c",
                    "time_frame": null, // Nullable
                    "title": null, // Nullable
                    "updated_at": null // Nullable
                }
                ...
            ],
            "updated_at": null,
            "updated_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e"
        }
    ],
    "error": null
}
```

To unsubscribe, the user can send a request to `/threads/unsubscribe` that has a null `id` field.

## `/threads/post`

When a user requests `/threads/post`, they are automatically subscribed to a channel for that thread.

**This endpoint is also used for continuous chatting. All other updates to a thread are sent through the [`/threads/update`](#threadsupdate) endpoint.**

### Request

```json
{
    "route": "/threads/post",
    "data": {
        "prompt": "show me my top customer",
        "dataset_id": null, // Nullable, if null, the dataset is auto-picked.
        "thread_id": null // Nullable, if null, a new thread is created.
    }
}
```

## Events/Responses

### `initializeThread`

Immediately after firing off the request, you will be sent the thread object back.

```json
{
    "route": "/threads/post",
    "event": "initializeThread",
    "payload": {
        "created_at": "2024-06-14T19:26:13.186043Z",
        "created_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
        "dashboard_id": null,
        "deleted_at": null,
        "folder_id": null,
        "id": "1e7d2339-69b6-4c49-bfa8-86fa9ad93287",
        "messages": [
            {
                "chart_config": null,
                "chart_recommendations": null,
                "code": null,
                "context": null,
                "created_at": "2024-06-14T19:26:13.186051Z",
                "data_metadata": null,
                "dataset_id": null,
                "deleted_at": null,
                "feedback": null,
                "id": "1469daa6-188e-466b-b35d-8ef580194f69",
                "message": "show me my top customer",
                "responses": null,
                "sent_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
                "sent_by_name": "Chad",
                "thread_id": "1e7d2339-69b6-4c49-bfa8-86fa9ad93287",
                "time_frame": null,
                "title": null,
                "updated_at": null,
                "status": "NotVerified"
            }
        ],
        "updated_at": null,
        "updated_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e"
    },
    "error": null
}
```

### `joinedThread`

Every time a user joins a thread, their 'presence' is sent via a 'joined' event.  This one is specific to the thread that was just fired off for the post.

```json
{
    "route": "/threads/post",
    "event": "joinedThread",
    "payload": {
        "email": "chad@buster.so",
        "id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
        "name": "Chad",
        "thread_id": "1e7d2339-69b6-4c49-bfa8-86fa9ad93287"
    },
    "error": null
}
```

This is by far the most complex series of events and responses.  It contains a series of `threadStepProgress` events, `threadStepCompletion` events, and `threadCheckpoint` events.

### Streaming Events

This event is pretty simple, it just consists of an event enum identifying what step we are working on.  Will always be sent in order.
The `event` field can be one of the following:

- `identifyingDataset`
- `identifyingTerms`
- `generatingSql`
- `fixingSql`
- `fetchingData`
- `generatingDataExplanation`
- `generatingTimeFrame`
- `generatingMetricTitle`
- `generatingDataSummary`

#### `identifyingDataset`

```json
{
    "route": "/threads/post",
    "event": "identifyingDataset",
    "payload": {
        "dataset": { // dataset is null if 'inProgress'
            "id": "5381a3ac-9671-4318-a8d3-3dc34b7d3c42",
            "name": "Sales Dataset"
        },
        "progress": "completed" // can be 'inProgress', 'completed', or 'failed'
    },
    "error": null
}
```

```json
{
    "route": "/threads/post",
    "event": "identifyingTerms",
    "payload": {
        "progress": "inProgress",
        "terms": null
    },
    "error": null
}
```

```json
// You should expect many of these chunks to roll in sequentially.
{
    "route": "/threads/post",
    "event": "generatingSql",
    "payload": {
        "progress": "inProgress",
        "sql_chunk": "WITH"
    },
    "error": null
}
```

```json
{
    "route": "/threads/post",
    "event": "fetchingData",
    "payload": {
        "data": [
            {
                "customer_name": "QUICK-Stop",
                "total_sales": 109545.80504002742
            }
        ],
        "progress": "completed"
    },
    "error": null
}
```

```json
{
    "route": "/threads/post",
    "event": "generatingDataSummary",
    "payload": {
        "data_summary": null,
        "data_summary_chunk": "some string of characters",
        "progress": "inProgress"
    },
    "sent_by": {
        "id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
        "name": "Chad"
    },
    "error": null,
    "send_back_to_user": true
}
```

```json
{
    "route": "/threads/post",
    "event": "generatingMetricTitle",
    "payload": {
        "metric_title": null,
        "metric_title_chunk": "some string of chars",
        "progress": "inProgress"
    },
    "sent_by": {
        "id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
        "name": "Chad"
    },
    "error": null,
    "send_back_to_user": true
}
```

```json
{
    "route": "/threads/post",
    "event": "generatingDataExplanation",
    "payload": {
        "data_explanation": null,
        "data_explanation_chunk": " some string of chars",
        "progress": "inProgress"
    },
    "sent_by": {
        "id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
        "name": "Chad"
    },
    "error": null,
    "send_back_to_user": true
}
```

```json
{
    "route": "/threads/post",
    "event": "generatingTimeFrame",
    "payload": {
        "progress": "inProgress",
        "time_frame": null,
        "time_frame_chunk": "some string of chars"
    },
    "sent_by": {
        "id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
        "name": "Chad"
    },
    "error": null,
    "send_back_to_user": true
}
```

To unsubscribe, the user can send a request to `/threads/unsubscribe` that has a null `id` field.

## `/threads/update`

When a user requests `/threads/update`, they are automatically subscribed to a channel for that thread.

### Request

```json
{
    "route": "/threads/update",
    "data": {
        "id": "3912181d-0f67-4b92-b340-37fa11dddc4c",
        "code": "SELECT * FROM sales WHERE date > '2024-06-11'", //Optional
        "title": "Sales Analysis", //Optional
        "feedback": "Positive", //Optional, can be 'Positive' or 'Negative'
        "status": "Verified", //Optional, can be 'Verified', 'NotVerified', or 'Rejected'
        "chart_config": { //Optional
            "type": "line",
            "x": "date",
            "y": "sales"
        }
    }
}
```

## Events/Responses

### `updateThreadState`

This response is sent when the thread state is updated.

```json
{
    "route": "/threads/update",
    "event": "updateThreadState",
    "payload": [
        {
            "created_at": "2024-06-17T16:53:34.852505Z",
            "created_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "dashboard_id": null,
            "deleted_at": null,
            "folder_id": null,
            "id": "b4e5642c-3b40-4730-afbd-47cc28f7bd2e",
            "messages": [
                {
                    "chart_config": {},
                    "chart_recommendations": {},
                    "code": "\nWITH yearly_sales AS (\n    SELECT \n        DISTINCT EXTRACT(YEAR FROM order_date) AS year,\n        SUM(total_sales_amount) AS total_sales\n    FROM \n        sales_summary\n    GROUP BY \n        EXTRACT(YEAR FROM order_date)\n)\nSELECT \n    year,\n    total_sales\nFROM \n    yearly_sales\nORDER BY \n    year;",
                    "context": null,
                    "created_at": "2024-06-17T16:53:34.852516Z",
                    "data_metadata": {
                        "column_count": 2,
                        "column_metadata": [
                            {
                                "name": "year",
                                "type": "float8"
                            },
                            {
                                "name": "total_sales",
                                "type": "float8"
                            }
                        ],
                        "row_count": 3
                    },
                    "dataset_id": "5381a3ac-9671-4318-a8d3-3dc34b7d3c42",
                    "deleted_at": null,
                    "feedback": null,
                    "id": "66b55ca3-3d05-488d-8628-f2f0c7f4fe7b",
                    "message": "show me total sales on a bar chart by year",
                    "responses": {
                        "messages": [
                            "The total sales have shown significant fluctuations over the years, with $142,063 in 2022, peaking at $591,339 in 2023, and then dropping to $450,651 in 2024. This indicates a sharp increase in sales in 2023 followed by a decline in 2024.",
                            "I pulled the total sales for each year by first calculating the yearly sales from the dataset. Then, I summed up the sales amounts for each year and organized them by year. Finally, I sorted the results by year to make it easy to create a bar chart showing total sales over time."
                        ]
                    },
                    "sent_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
                    "sent_by_name": "Chad",
                    "thread_id": "b4e5642c-3b40-4730-afbd-47cc28f7bd2e",
                    "time_frame": null,
                    "title": "This is a test title",
                    "updated_at": null,
                    "status": "notVerified"
                }
            ],
            "title": "Total Sales by Year",
            "updated_at": null,
            "updated_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e"
        }
    ],
    "error": null
}
```

**This endpoint is not used for continuous chatting.  Only for updates to attributes about the thread.**

## `/threads/delete`

When a user requests `/threads/delete` they are not subscribed to any channels.  A message is sent to all subsribers that the thread has been deleted.  This req also unsubscribes the user from the thread subscription.

### Request

```json
{
    "route": "/threads/delete",
    "data": {
        "id": ["3912181d-0f67-4b92-b340-37fa11dddc4c"] // Can be many ids
    }
}
```

### Response

### `deleteThreadState`

```json
{
    "route": "/threads/delete",
    "event": "deleteThreadState",
    "payload": ["3912181d-0f67-4b92-b340-37fa11dddc4c"], // Can be many ids
    "error": null
}
```

## `/threads/search`

This is used to search all of the threads that the user has access to via hybrid search. Max results is 10.

### Request

```json
{
    "route": "/threads/search",
    "payload": {
        "prompt": "what were our total sales"
    }
}
```

### Response

```json
{
    "route": "/threads/search",
    "event": "searchThreads",
    "payload": [
        {
            "chart_config": {},
            "chart_recommendations": {},
            "code": null,
            "context": {
              "steps": [
                {
                  "identifyingDataset": {
                    "dataset": {
                      "id": "5381a3ac-9671-4318-a8d3-3dc34b7d3c42",
                      "name": "Sales Dataset"
                    },
                    "progress": "completed",
                    "threadId": "6c7668cc-b82f-4b38-9a6a-965049d4aa1c",
                    "messageId": "045b7723-d758-4c3d-9b3c-f912e8b0083a"
                  }
                },
                {
                  "generatingSql": {
                    "sql": "\nWITH monthly_sales AS (\n    SELECT \n        DATE_TRUNC('month', order_date) AS month,\n        SUM(total_sales_amount) AS total_sales\n    FROM \n        (SELECT DISTINCT order_id, order_date, total_sales_amount FROM sales_summary) AS deduped_sales\n    GROUP BY \n        DATE_TRUNC('month', order_date)\n)\nSELECT \n    TO_CHAR(month, 'YYYY-MM') AS month,\n    total_sales\nFROM \n    monthly_sales\nORDER BY \n    total_sales DESC\nLIMIT 1;",
                    "progress": "completed",
                    "sqlChunk": null,
                    "threadId": "6c7668cc-b82f-4b38-9a6a-965049d4aa1c",
                    "messageId": "045b7723-d758-4c3d-9b3c-f912e8b0083a"
                  }
                },
                {
                  "fetchingData": {
                    "data": null,
                    "progress": "completed",
                    "threadId": "6c7668cc-b82f-4b38-9a6a-965049d4aa1c",
                    "messageId": "045b7723-d758-4c3d-9b3c-f912e8b0083a"
                  }
                }
              ]
            },
            "created_at": "2024-06-18T03:47:59.124284Z",
            "data_metadata": null,
            "dataset_id": "5381a3ac-9671-4318-a8d3-3dc34b7d3c42",
            "deleted_at": null,
            "feedback": null,
            "id": "933cdaf8-ae2a-40aa-a386-942fed283daa",
            "message": "Show me our total sales from the last month.",
            "responses": null,
            "sent_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "status": "verified",
            "thread_id": "3912181d-0f67-4b92-b340-37fa11dddc4c",
            "time_frame": null,
            "title": "Total Sales From Last Month",
            "updated_at": null
        },
        {
            "chart_config": {},
            "chart_recommendations": {},
            "code": null,
            "context": {
              "steps": [
                {
                  "identifyingDataset": {
                    "dataset": {
                      "id": "5381a3ac-9671-4318-a8d3-3dc34b7d3c42",
                      "name": "Sales Dataset"
                    },
                    "progress": "completed",
                    "threadId": "6c7668cc-b82f-4b38-9a6a-965049d4aa1c",
                    "messageId": "045b7723-d758-4c3d-9b3c-f912e8b0083a"
                  }
                },
                {
                  "generatingSql": {
                    "sql": "\nWITH monthly_sales AS (\n    SELECT \n        DATE_TRUNC('month', order_date) AS month,\n        SUM(total_sales_amount) AS total_sales\n    FROM \n        (SELECT DISTINCT order_id, order_date, total_sales_amount FROM sales_summary) AS deduped_sales\n    GROUP BY \n        DATE_TRUNC('month', order_date)\n)\nSELECT \n    TO_CHAR(month, 'YYYY-MM') AS month,\n    total_sales\nFROM \n    monthly_sales\nORDER BY \n    total_sales DESC\nLIMIT 1;",
                    "progress": "completed",
                    "sqlChunk": null,
                    "threadId": "6c7668cc-b82f-4b38-9a6a-965049d4aa1c",
                    "messageId": "045b7723-d758-4c3d-9b3c-f912e8b0083a"
                  }
                },
                {
                  "fetchingData": {
                    "data": null,
                    "progress": "completed",
                    "threadId": "6c7668cc-b82f-4b38-9a6a-965049d4aa1c",
                    "messageId": "045b7723-d758-4c3d-9b3c-f912e8b0083a"
                  }
                }
              ]
            },
            "created_at": "2024-06-18T03:47:59.124285Z",
            "data_metadata": null,
            "dataset_id": "5381a3ac-9671-4318-a8d3-3dc34b7d3c42",
            "deleted_at": null,
            "feedback": null,
            "id": "3028e20b-22c1-4c47-9e6a-b1c8eba532fa",
            "message": "this is great, can you break that down week by week",
            "responses": null,
            "sent_by": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
            "status": "notVerified",
            "thread_id": "3912181d-0f67-4b92-b340-37fa11dddc4c",
            "time_frame": null,
            "title": "Total Sale From Last Month Broken Down By Week",
            "updated_at": null
        }
    ],
    "sent_by": {
        "id": "c2dd64cd-f7f3-4884-bc91-d46ae431901e",
        "name": "Chad"
    },
    "error": null,
    "send_back_to_user": true
}
```

## `/threads/unsubscribe`

When a user requests `/threads/unsubscribe` they unsubscribe from all *or one* threads they are subscribed to.

### Request

```json
{
    "route": "/threads/unsubscribe",
    "data": {
        "id": "3912181d-0f67-4b92-b340-37fa11dddc4c" // This is the `thread_id` of the thread to unsubscribe from.
        // Nullable, if null, unsubscribes from all threads.
    }
}
```

### Response

```json
{
    "route": "/threads/unsubscribe",
    "event": "unsubscribed",
    "payload": [
        {
            "id": "3912181d-0f67-4b92-b340-37fa11dddc4c" // This is the `thread_id` that was unsubscribed from.
            // Nullable, if null, all threads were unsubscribed from.
        }
    ]
}
```
