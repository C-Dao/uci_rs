mod storage;
mod utils;
mod webserver;

use webserver::App;

fn main() {
    let app = App::new();
    app.listen().unwrap();
}
