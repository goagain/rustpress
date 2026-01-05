use crate::{
    dto::CreatePostRequest, plugin::exports::rustpress::plugin::post_hooks::OnPostPublishedData,
};

impl From<OnPostPublishedData> for CreatePostRequest {
    fn from(data: OnPostPublishedData) -> Self {
        CreatePostRequest {
            title: data.title,
            content: data.content,
            category: data.category.clone(),
            author_id: data.author_id,
        }
    }
}

impl From<CreatePostRequest> for OnPostPublishedData {
    fn from(data: CreatePostRequest) -> Self {
        OnPostPublishedData {
            title: data.title,
            content: data.content,
            category: data.category.clone(),
            author_id: data.author_id,
        }
    }
}
