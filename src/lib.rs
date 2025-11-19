use serde::Deserialize;
use wit_bindgen::generate;

generate!({
    path: "wit",
});

use exports::jsonplaceholder::api::jsonplaceholder_api::Guest as JsonplaceholderApi;
use exports::jsonplaceholder::api::jsonplaceholder_api::NotFoundError;
use wasi::http::types::*;

use crate::exports::jsonplaceholder::api::jsonplaceholder_api::{
    Address, Album, Comment, Company, Geo, Photo, Post, Todo, User,
};

const BASE: &str = "https://jsonplaceholder.typicode.com";

/// Generic HTTP GET JSON
fn fetch_json<T: for<'a> Deserialize<'a>>(path: &str) -> Result<T, ()> {
    // construct URL
    let url = format!("{BASE}{path}");

    let req = OutgoingRequest::new(Fields::new());
    req.set_method(&Method::Get).map_err(|_| ())?;
    req.set_path_with_query(Some(&url)).map_err(|_| ())?;

    let body = req.body().unwrap(); // no body for GET
    drop(body);

    let res_fut = wasi::http::outgoing_handler::handle(req, None).map_err(|_| ())?;
    let res = match res_fut.get() {
        Some(Ok(Ok(r))) => r,
        _ => return Err(()),
    };
    if res.status() != 200 {
        return Err(());
    }

    // read body
    let incoming_body = res.consume().unwrap();
    let stream = incoming_body.stream().unwrap();
    let mut bytes = Vec::new();
    loop {
        let chunk = stream.read(4096).map_err(|_| ())?;
        if chunk.is_empty() {
            break;
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| ())
}

//
// DATA MODELS FOR SERDE
//

#[derive(Deserialize)]
struct GeoSerde {
    lat: String,
    lng: String,
}

impl From<GeoSerde> for Geo {
    fn from(g: GeoSerde) -> Self {
        Geo {
            lat: g.lat,
            lng: g.lng,
        }
    }
}

#[derive(Deserialize)]
struct AddressSerde {
    street: String,
    suite: String,
    city: String,
    zipcode: String,
    geo: GeoSerde,
}

impl From<AddressSerde> for Address {
    fn from(a: AddressSerde) -> Self {
        Address {
            street: a.street,
            suite: a.suite,
            city: a.city,
            zipcode: a.zipcode,
            geo: a.geo.into(),
        }
    }
}

#[derive(Deserialize)]
struct CompanySerde {
    name: String,
    #[serde(rename = "catchPhrase")]
    catch_phrase: String,
    bs: String,
}

impl From<CompanySerde> for Company {
    fn from(c: CompanySerde) -> Self {
        Company {
            name: c.name,
            catch_phrase: c.catch_phrase,
            bs: c.bs,
        }
    }
}

#[derive(Deserialize)]
struct PostSerde {
    id: u64,
    #[serde(rename = "userId")]
    user_id: u64,
    title: String,
    body: String,
}

impl From<PostSerde> for Post {
    fn from(p: PostSerde) -> Self {
        Post {
            id: p.id,
            user_id: p.user_id,
            title: p.title,
            body: p.body,
        }
    }
}

#[derive(Deserialize)]
struct UserSerde {
    id: u64,
    name: String,
    username: String,
    email: String,
    phone: String,
    website: String,
    company: CompanySerde,
    address: AddressSerde,
}

impl From<UserSerde> for User {
    fn from(u: UserSerde) -> Self {
        User {
            username: u.username,
            id: u.id,
            name: u.name,
            email: u.email,
            phone: u.phone,
            website: u.website,
            company: u.company.into(),
            address: u.address.into(),
        }
    }
}

#[derive(Deserialize)]
struct CommentSerde {
    id: u64,
    #[serde(rename = "postId")]
    post_id: u64,
    name: String,
    email: String,
    body: String,
}

impl From<CommentSerde> for Comment {
    fn from(c: CommentSerde) -> Self {
        Comment {
            id: c.id,
            post_id: c.post_id,
            name: c.name,
            email: c.email,
            body: c.body,
        }
    }
}

#[derive(Deserialize)]
struct AlbumSerde {
    id: u64,
    #[serde(rename = "userId")]
    user_id: u64,
    title: String,
}

impl From<AlbumSerde> for Album {
    fn from(a: AlbumSerde) -> Self {
        Album {
            id: a.id,
            user_id: a.user_id,
            title: a.title,
        }
    }
}

#[derive(Deserialize)]
struct PhotoSerde {
    id: u64,
    #[serde(rename = "albumId")]
    album_id: u64,
    title: String,
    url: String,
    #[serde(rename = "thumbnailUrl")]
    thumbnail_url: String,
}

impl From<PhotoSerde> for Photo {
    fn from(p: PhotoSerde) -> Self {
        Photo {
            id: p.id,
            album_id: p.album_id,
            thumbnail_url: p.thumbnail_url,
            title: p.title,
            url: p.url,
        }
    }
}

#[derive(Deserialize)]
struct TodoSerde {
    id: u64,
    #[serde(rename = "userId")]
    user_id: u64,
    title: String,
    completed: bool,
}

impl From<TodoSerde> for Todo {
    fn from(t: TodoSerde) -> Self {
        Todo {
            id: t.id,
            user_id: t.user_id,
            title: t.title,
            completed: t.completed,
        }
    }
}

//
// IMPLEMENTATION OF THE WIT INTERFACE
//

struct ApiImpl;

impl JsonplaceholderApi for ApiImpl {
    //
    // posts
    fn get_posts(user_id: u64) -> Vec<exports::jsonplaceholder::api::jsonplaceholder_api::Post> {
        fetch_json::<Vec<PostSerde>>(&format!("/posts?userId={user_id}"))
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.into())
            .collect()
    }

    fn get_post(
        id: u64,
    ) -> Result<exports::jsonplaceholder::api::jsonplaceholder_api::Post, NotFoundError> {
        fetch_json::<PostSerde>(&format!("/posts/{id}"))
            .map(|p| p.into())
            .map_err(|_| NotFoundError { message: "Not found".to_string() })
    }

    fn get_post_comments(
        id: u64,
    ) -> Result<Vec<exports::jsonplaceholder::api::jsonplaceholder_api::Comment>, NotFoundError>
    {
        fetch_json::<Vec<CommentSerde>>(&format!("/posts/{id}/comments"))
            .map(|v| v.into_iter().map(|c| c.into()).collect())
            .map_err(|_| NotFoundError { message: "Not found".to_string() })
    }

    fn get_comments(
        id: Option<u64>,
        post_id: Option<u64>,
    ) -> Vec<exports::jsonplaceholder::api::jsonplaceholder_api::Comment> {
        let mut q = vec![];
        if let Some(i) = id {
            q.push(format!("id={i}"));
        }
        if let Some(p) = post_id {
            q.push(format!("postId={p}"));
        }

        let query = if q.is_empty() {
            "".to_string()
        } else {
            format!("?{}", q.join("&"))
        };

        fetch_json::<Vec<CommentSerde>>(&format!("/comments{query}"))
            .unwrap_or_default()
            .into_iter()
            .map(|c| c.into())
            .collect()
    }

    fn get_comment(
        id: u64,
    ) -> Result<exports::jsonplaceholder::api::jsonplaceholder_api::Comment, NotFoundError> {
        fetch_json::<CommentSerde>(&format!("/comments/{id}"))
            .map(|c| c.into())
            .map_err(|_| NotFoundError { message: "Not found".to_string() })
    }

    fn get_albums(
        id: Option<u64>,
        user_id: Option<u64>,
    ) -> Vec<exports::jsonplaceholder::api::jsonplaceholder_api::Album> {
        let mut q = vec![];
        if let Some(i) = id {
            q.push(format!("id={i}"));
        }
        if let Some(u) = user_id {
            q.push(format!("userId={u}"));
        }

        let query = if q.is_empty() {
            "".to_string()
        } else {
            format!("?{}", q.join("&"))
        };

        fetch_json::<Vec<AlbumSerde>>(&format!("/albums{query}"))
            .unwrap_or_default()
            .into_iter()
            .map(|a| a.into())
            .collect()
    }

    fn get_album(
        id: u64,
    ) -> Result<exports::jsonplaceholder::api::jsonplaceholder_api::Album, NotFoundError> {
        fetch_json::<AlbumSerde>(&format!("/albums/{id}"))
            .map(|a| a.into())
            .map_err(|_| NotFoundError { message: "Not found".to_string() })
    }

    fn get_album_photos(
        id: u64,
    ) -> Result<Vec<exports::jsonplaceholder::api::jsonplaceholder_api::Photo>, NotFoundError> {
        fetch_json::<Vec<PhotoSerde>>(&format!("/albums/{id}/photos"))
            .map(|v| v.into_iter().map(|p| p.into()).collect())
            .map_err(|_| NotFoundError { message: "Not found".to_string() })
    }

    fn get_photos(
        id: Option<u64>,
        album_id: Option<u64>,
    ) -> Vec<exports::jsonplaceholder::api::jsonplaceholder_api::Photo> {
        let mut q = vec![];
        if let Some(i) = id {
            q.push(format!("id={i}"));
        }
        if let Some(a) = album_id {
            q.push(format!("albumId={a}"));
        }

        let query = if q.is_empty() {
            "".to_string()
        } else {
            format!("?{}", q.join("&"))
        };

        fetch_json::<Vec<PhotoSerde>>(&format!("/photos{query}"))
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.into())
            .collect()
    }

    fn get_photo(
        id: u64,
    ) -> Result<exports::jsonplaceholder::api::jsonplaceholder_api::Photo, NotFoundError> {
        fetch_json::<PhotoSerde>(&format!("/photos/{id}"))
            .map(|p| p.into())
            .map_err(|_| NotFoundError { message: "Not found".to_string() })
    }

    fn get_todos(
        id: Option<u64>,
        user_id: Option<u64>,
    ) -> Vec<exports::jsonplaceholder::api::jsonplaceholder_api::Todo> {
        let mut q = vec![];
        if let Some(i) = id {
            q.push(format!("id={i}"));
        }
        if let Some(u) = user_id {
            q.push(format!("userId={u}"));
        }

        let query = if q.is_empty() {
            "".to_string()
        } else {
            format!("?{}", q.join("&"))
        };

        fetch_json::<Vec<TodoSerde>>(&format!("/todos{query}"))
            .unwrap_or_default()
            .into_iter()
            .map(|t| t.into())
            .collect()
    }

    fn get_todo(
        id: u64,
    ) -> Result<exports::jsonplaceholder::api::jsonplaceholder_api::Todo, NotFoundError> {
        fetch_json::<TodoSerde>(&format!("/todos/{id}"))
            .map(|t| t.into())
            .map_err(|_| NotFoundError { message: "Not found".to_string() })
    }

    fn get_users(
        id: Option<u64>,
        email: Option<String>,
    ) -> Vec<exports::jsonplaceholder::api::jsonplaceholder_api::User> {
        let mut q = vec![];
        if let Some(i) = id {
            q.push(format!("id={i}"));
        }
        if let Some(e) = email {
            q.push(format!("email={e}"));
        }

        let query = if q.is_empty() {
            "".to_string()
        } else {
            format!("?{}", q.join("&"))
        };

        fetch_json::<Vec<UserSerde>>(&format!("/users{query}"))
            .unwrap_or_default()
            .into_iter()
            .map(|u| u.into())
            .collect()
    }

    fn get_user(
        id: u64,
    ) -> Result<exports::jsonplaceholder::api::jsonplaceholder_api::User, NotFoundError> {
        fetch_json::<UserSerde>(&format!("/users/{id}"))
            .map(|u| u.into())
            .map_err(|_| NotFoundError { message: "Not found".to_string() })
    }
}

__export_jsonplaceholder_impl!(ApiImpl);
