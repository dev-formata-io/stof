
from flask import Flask, request, Response
from pystof import Doc


app = Flask(__name__)
doc = Doc()
request_root = None
response_root = None


# The Stof required to make our server work.
# This includes a Request root, Response root, and a cleanup function.
base_path = 'src/py/examples/server/base.stof'
base_file = open(base_path, 'r')
BASE = base_file.read()
base_file.close()


# The Stof API our server offers for clients.
api_path = 'src/py/examples/server/api.stof'
api_file = open(api_path, 'r')
API = api_file.read()
api_file.close()


def init_doc():
    global request_root, response_root, BASE, API
    doc.parse(BASE)
    doc.parse(API)
    request_root = doc.get('Request', None)
    response_root = doc.get('Response', None)


@app.post('/')
def stof_entry():
    global doc, request_root, response_root

    # import into "Reqeust" root - a specific, known location in our document
    doc.binary_import(request.get_data(), 'bstf', request_root)
    doc.run('server')

    # export json from "Response" root - a specific, know location in our document
    res_json = doc.string_export('json', response_root)
    doc.run('cleanup') # remove any Request & Response data for the next pass
    request_root = doc.get('Request', None)
    response_root = doc.get('Response', None)

    return Response(res_json, mimetype='application/json')


if __name__ == '__main__':
    init_doc()
    app.run(debug=True)
