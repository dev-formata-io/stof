# Python Server
Just a quick example to show Stof moving between a client & server. The client sends some stof to the server. The server uses it to manipulate a doc it controls and create a JSON response.

> Note: in a production env make sure to sanbox the Stof graph. Remove the file system lib functions, etc. This has already been done for JS (no system access by default), but for python the sandboxing features are still in progress.

## Server
Receives a binary request body in "bstf" format (binary stof). Could use whatever format you'd like here, but bstf or stof are good choices for us because they can contain functions the server can call.

The server then calls all #[server] functions in the request body, allowing the request to manipulate a Stof document it controls.

The client will create their own response JSON by adding data to the "Response" root of the document.

## Client
Just sends a bstf request to the server and prints the JSON response.

## To run
1. Get python if needed
2. pip install stof
3. pip install flask
4. Start server: python src/py/examples/server/server.py
5. Run client: python src/py/examples/server/client.py
6. See JSON output and be happy