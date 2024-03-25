# 

## What is this?

I made this as an example of a messages microservice. It is a simple REST API that allows you to create, read, update, and delete messages.

It uses ScyllaDB as the database and Rust as the programming language.

It's based on these 2 articles from Discord's engineering blog:
- [How Discord Stores Trillions of Messages](https://discord.com/blog/how-discord-stores-trillions-of-messages)
- [Why Discord is Switching from Go to Rust](https://discord.com/blog/why-discord-is-switching-from-go-to-rust)

```sql
CREATE TABLE messages (
   channel_id bigint,
   bucket int,
   message_id bigint,
   author_id bigint,
   content text,
   PRIMARY KEY ((channel_id, bucket), message_id)
) WITH CLUSTERING ORDER BY (message_id DESC);
```

## How to run

```bash
docker compose build && docker compose up
```

## How to use

### Create a message

```bash
curl -X POST -H "Content-Type: application/json" http://localhost:3000/messages -d '{"channel_id": 1, "message_id": 14, "author_id": 1, "content": "Hello, world!"}'
```

### Read a message

```bash
curl -X GET -H "Content-Type: application/json" http://localhost:3000/messages -d '{"channel_id": 1}'
```

### Update a message

```bash
curl -X PUT -H "Content-Type: application/json" http://localhost:3000/messages -d '{"channel_id": 1, "message_id": 14, "author_id": 1, "content": "Hello, world! (edited)"}'
```

### Delete a message

```bash
curl -X DELETE -H "Content-Type: application/json" http://localhost:3000/messages -d '{"channel_id": 1, "message_id": 14}'
```