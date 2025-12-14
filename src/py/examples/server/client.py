
from pystof import Doc

stof = """

request: {
    #[server]
    /// Runs on the server, using any APIs the server provides.
    /// Anything we want sent back to us should go in the "Response" root.
    fn called_on_remote_server() {
        <Point>.plot(1.5m, 2.5cm);
        <Point>.plot(0.5km, 0.5m);
        <Point>.plot(1.25cm, 3m);
        <Point>.plot(1mm, 1um);

        <Point>.translate_all(300cm, -0.2m);
        Response.points = <Point>.points();
    }
}

#[main]
/// Runs here locally, sends the request to the server as BSTF (could use Stof too...).
fn main() {
    const bytes_body = blobify('bstf', self.request);
    const headers = { 'Content-Type': 'application/bstf' };
    const res = await Http.fetch('http://localhost:5000', 'post', bytes_body, headers);
    pln(Http.parse(res, new {}));
}

"""

def main():
    global stof
    doc = Doc()
    doc.parse(stof)
    doc.run()

if __name__ == '__main__':
    main()
