mod classification;
mod server;

use server::redirect;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    redirect()
}
