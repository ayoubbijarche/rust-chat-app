#[macro_use] extern crate rocket;
use rocket::{form::Form, fs::{FileServer, relative}, response::stream::{Event, EventStream}, serde::{Deserialize, Serialize}, tokio::sync::broadcast::{channel, error::RecvError, Sender}, Shutdown, State};
use rocket::tokio::select;

#[derive(Debug , Clone , FromForm , Serialize , Deserialize)]
#[serde(crate = "rocket::serde")]
struct  Messages{
    #[field(validate = len(..8))]
    pub username : String,
    #[field(validate = len(..20))]
    pub room : String,
    #[field(validate = len(..30))]
    pub message : String
}

#[post("/chat" , data="<form>")]
fn post(form : Form<Messages> , queue : &State<Sender<Messages>>){
    let _outcome = queue.send(form.into_inner());
}

#[get("/event")]
async fn events(queue : &State<Sender<Messages>> ,  mut end : Shutdown)->EventStream![]{
    let mut received = queue.subscribe();
    EventStream!{
        loop{
            let msg = select! {
                msg = received.recv() => match msg{
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    }

}


#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<Messages>(1024).0)
        .mount("/", routes![post , events])
        .mount("/", FileServer::from(relative!("template")))
}
