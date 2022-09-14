mod utils;
mod webserver;
mod storage;

use webserver::App;

fn main() {
    let app = App::new();
    app.listen().unwrap();
}
