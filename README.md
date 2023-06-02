# Microblog API Setup and Usage Guide

This guide provides instructions on how to compile, setup, and manage the Microblog API server. Follow these steps to get started.

## Prerequisites
- V programming language (Vlang) installed on your machine.
- SQLite database system installed.
- Basic understanding of command line usage.

## Token Provisioning and Security

In the Microblog API, tokens play a crucial role in securing and controlling access to various operations. The token generation process involves an admin provisioning the tokens through tertiary means, ensuring an extra layer of security. Let's explore this process and its significance.

### Token Generation

When an admin needs to generate a token, they must perform the following steps:

1. Use tertiary means: The admin uses an external method or system, separate from the Microblog API, to generate a token. This can involve procedures like executing a command on a different server, generating tokens offline, or using dedicated token generation services. By employing tertiary means, the admin ensures that token generation is decoupled from the immediate Microblog API environment, adding an extra layer of security.

2. Admin Password Requirement: To enhance security, the token generation route is protected with an admin password. This means that only authorized individuals who possess the admin password can generate tokens. When making a request to generate a token, the admin must provide the correct admin password along with the request payload.

3. Token Permissions: Each token generated by the admin contains additional information about its permissions and capabilities. This allows the Microblog API to validate the token's relevance to specific operations. For example, certain tokens may have read-only access to microblogs, while others may have full CRUD (Create, Read, Update, Delete) capabilities. By incorporating permissions into tokens, the API can enforce fine-grained access control.

### Token Usage and Security

Once tokens are generated, they can be used by clients to authenticate and authorize their requests to the Microblog API. The API employs a validation mechanism to ensure that tokens are valid and have not been used before.

1. Validation: When a request with a token is received, the Microblog API validates the token against the stored tokens. It checks if the token exists, is unused, and matches the required permissions for the requested operation. This validation process ensures that only valid and authorized tokens are accepted.

2. One-Time Use: Tokens generated by the admin are designed to be used only once. Once a token is successfully validated, it is marked as used, preventing its reuse for subsequent requests. This adds an extra layer of security by mitigating the risk of token reuse or unauthorized access.

By implementing these token provisioning and security measures, the Microblog API promotes secure authentication and authorization, safeguarding the microblogging system and its resources.

Please note that the exact implementation details of token provisioning and security may vary depending on your specific requirements and the external token generation mechanisms you choose to employ.

With these changes, the token generation route will require the admin password to be provided in the request body. If the provided password matches the configured admin password, the token will be generated. Otherwise, an unauthorized response will be returned.

Remember to securely manage and protect the admin password to maintain the security of your application.

## Compilation

1. Clone the repository to your local machine:

   ```bash
   git clone https://github.com/SaulDoesCode/mb.git
   ```

2. Change into the project directory:

   ```bash
   cd microblog-api
   ```

3. Compile the Vlang source code to create an executable:

   ```bash
   v -prod .
   ```

   This will generate an executable file named `microblog-api` in the current directory.

## Database Setup

1. Create a new SQLite database file:

   ```bash
   sqlite3 microblog.db
   ```

2. Create the necessary tables in the database:

   ```sql
   CREATE TABLE IF NOT EXISTS nodes (id TEXT PRIMARY KEY, value TEXT);
   CREATE TABLE IF NOT EXISTS relations (name TEXT, from_id TEXT, to_id TEXT);
   ```

3. Configure the database path in the `main()` function of the source code:

   ```v
   api.rhyzome.open("microblog.db") or { panic(err) }
   ```

## Starting the Server

1. Start the Microblog API server by running the compiled executable:

   ```bash
   ./microblog-api
   ```

2. The server will start and listen for incoming requests on `http://localhost:8080`.

## API Usage

### Create a Microblog

To create a new microblog, send a `POST` request to `/microblogs` endpoint:

```bash
curl -X POST -H "Content-Type: application/json" -d '{"text": "This is my first microblog!"}' http://localhost:8080/microblogs
```

### Get All Microblogs

To retrieve all microblogs, send a `GET` request to `/microblogs` endpoint:

```bash
curl http://localhost:8080/microblogs
```

### Get a Microblog

To retrieve a specific microblog by its ID, send a `GET` request to `/microblogs/{id}` endpoint:

```bash
curl http://localhost:8080/microblogs/{id}
```

### Delete a Microblog

To delete a specific microblog by its ID, send a `DELETE` request to `/microblogs/{id}` endpoint:

```bash
curl -X DELETE http://localhost:8080/microblogs/{id}
```

### Create a Relation

To create a relation between two microblogs, send a `POST` request to `/microblogs/{id}/relations/{relation_name}` endpoint:

```bash
curl -X POST -H "Content-Type: application/json" -d '{"related_id": "{related_microblog_id}"}' http://localhost:8080/microblogs/{id}/relations/{relation_name}
```

### Delete a Relation

To delete a specific relation associated with a microblog, send a `DELETE` request to `/microblogs/{id}/relations/{relation_name}` endpoint:

```bash
curl -X DELETE http://localhost:8080/microblogs/{id}/relations/{relation_name}
```

### Generate an Admin Token

To generate an admin access token, send a `POST` request to `/tokens` endpoint with the admin password:

```bash
curl -X POST -H "Content-Type: application/json" -d '{"password": "{admin_password}"}' http://localhost:8080/tokens
```

## Done 
yay!

# Microblog API Documentation

## Introduction
The Microblog API allows users to create, retrieve, update, and delete microblogs. It provides a simple and lightweight interface for managing microblogging data. This documentation outlines the available endpoints and their functionalities.

## API Endpoints

### `GET /microblogs`
Retrieves a list of all microblogs.

### `POST /microblogs`
Creates a new microblog.

Request Body:
- `text` (string): The content of the microblog.

### `GET /microblogs/{id}`
Retrieves a specific microblog by its ID.

### `DELETE /microblogs/{id}`
Deletes a specific microblog by its ID.

### `GET /microblogs/{id}/relations`
Retrieves all relations associated with a specific microblog.

### `POST /microblogs/{id}/relations/{relation_name}`
Creates a new relation between two microblogs.

Request Body:
- `related_id` (string): The ID of the microblog to relate to.

### `DELETE /microblogs/{id}/relations/{relation_name}`
Deletes a specific relation associated with a microblog.

### `POST /tokens`
Generates a new access token for administrative purposes.

Request Body:
- `password` (string): The admin password required to generate the token.

## Authentication
Access to certain API endpoints requires authentication using an access token. The token can be obtained by calling the `/tokens` endpoint with the correct admin password. Once obtained, the token should be included in the `Authorization` header of subsequent requests.

## Error Handling
If an error occurs, the API will respond with an appropriate HTTP status code and a JSON response containing an error message.

# Admin Password, Change it!!!

To modify the Vlang script and change the admin password for the token generation route, follow these instructions:

1. Open the Vlang script file in a text editor of your choice.

2. Locate the line `` admin_password: "your_admin_password_here" `` should currently be line 178

3. Change the password.

4. Save the modified Vlang script file

5. run and enjoy, you can change it again ``vlang -prod mb.v``

## Conclusion
The Microblog API provides a straightforward and efficient way to manage microblogging data. By leveraging the Rhyzome database and a token-based authentication system, it ensures secure and reliable access to microblogs.
