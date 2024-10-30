#[macro_use] extern crate rocket;

use rocket::form::Form;
use rocket::response::content::RawHtml;
use rocket::http::Status;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;


#[derive(Debug, Clone)]
struct BlogPost {
    id: usize,
    title: String,
    content: String,
}

type PostsState = Arc<Mutex<HashMap<usize, BlogPost>>>;

#[launch]
fn rocket() -> _ {
    let posts: PostsState = Arc::new(Mutex::new(HashMap::new()));
    rocket::build()
        .manage(posts)
        .mount("/", routes![index, create_post, edit_post, update_post, delete_post, get_post])
}

#[get("/")]
fn index(posts: &rocket::State<PostsState>) -> RawHtml<String> {
    let posts = posts.lock().unwrap();
    let mut content = String::new();

    for post in posts.values() {
        content.push_str(&format!(
            "<h2>{}</h2><p>{}</p><a href=\"/post/{}/edit\">Edit</a> | <a href=\"/post/{}/delete\">Delete</a><hr>",
            post.title, post.content, post.id, post.id
        ));
    }

    if content.is_empty() {
        content.push_str("<h3>No posts available.</h3>");
    }

    let form = r#"
        <form action="/create" method="post">
            <label for="title">Title:</label><br>
            <input type="text" id="title" name="title" required><br>
            <label for="content">Content:</label><br>
            <textarea id="content" name="content" required></textarea><br>
            <input type="submit" value="Submit">
        </form>
    "#;

    RawHtml(format!("<h1>Blog Posts</h1>{}{}", content, form))
}

#[derive(FromForm)]
struct NewPost<'r> {
    title: &'r str,
    content: &'r str,
}

#[post("/create", data = "<new_post>")]
fn create_post(new_post: Form<NewPost>, posts: &rocket::State<PostsState>) -> RawHtml<String> {
    let mut posts = posts.lock().unwrap();
    let id = posts.len() + 1; 
    let new_blog_post = BlogPost {
        id,
        title: new_post.title.to_string(),
        content: new_post.content.to_string(),
    };
    posts.insert(id, new_blog_post);
    RawHtml(format!("Post created with ID: {}", id))
}

#[get("/post/<id>")]
fn get_post(id: usize, posts: &rocket::State<PostsState>) -> Option<RawHtml<String>> {
    let posts = posts.lock().unwrap();
    posts.get(&id).map(|post| {
        RawHtml(format!("<h1>{}</h1><p>{}</p>", post.title, post.content))
    })
}

#[get("/post/<id>/edit")]
fn edit_post(id: usize, posts: &rocket::State<PostsState>) -> Option<RawHtml<String>> {
    let posts = posts.lock().unwrap();
    posts.get(&id).map(|post| {
        RawHtml(format!(
            r#"
            <h1>Edit Post</h1>
            <form action="/post/{}/update" method="post">
                <label for="title">Title:</label><br>
                <input type="text" id="title" name="title" value="{}" required><br>
                <label for="content">Content:</label><br>
                <textarea id="content" name="content" required>{}</textarea><br>
                <input type="submit" value="Update">
            </form>
            "#,
            post.id, post.title, post.content
        ))
    })
}

#[post("/post/<id>/update", data = "<new_post>")]
fn update_post(id: usize, new_post: Form<NewPost>, posts: &rocket::State<PostsState>) -> Result<RawHtml<String>, Status> {
    let mut posts = posts.lock().unwrap();
    if let Some(post) = posts.get_mut(&id) {
        post.title = new_post.title.to_string();
        post.content = new_post.content.to_string();
        return Ok(RawHtml(format!("Post updated with ID: {}", id)));
    }
    Err(Status::NotFound)
}

#[get("/post/<id>/delete")]
fn delete_post(id: usize, posts: &rocket::State<PostsState>) -> Result<RawHtml<String>, Status> {
    let mut posts = posts.lock().unwrap();
    if posts.remove(&id).is_some() {
        Ok(RawHtml(format!("Post with ID: {} has been deleted.", id)))
    } else {
        Err(Status::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::blocking::Client;

    #[test]
    fn test_index() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/").dispatch();
        assert_eq!(response.status(), rocket::http::Status::Ok);
        assert!(response.into_string().unwrap().contains("Blog Posts"));
    }
}
