use crate::my_error::MyError;


#[derive(Debug)]
#[derive(Clone)]
pub struct Item{
    //see: https://www.rssboard.org/rss-specification#hrelementsOfLtitemgt
    pub title: String,
    pub link: String,
    pub description: String,
}

impl Item{
    pub fn new(title: String, link: String, description: String,) -> Self{
        Self {title:title, link:link, description:description}
    }
}

pub struct ItemBuilder{
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
}

impl ItemBuilder{
    pub fn new() -> Self{
        Self {title:None, link:None, description:None}
    }
    pub fn set_title(&mut self, title: &String){
        self.title = Some(title.clone());
    }
    pub fn set_link(&mut self, link: &String){
        self.link = Some(link.clone());
    }
    pub fn set_description(&mut self,description: &String){
        self.description = Some(description.clone());
    }
    pub fn ready(&self) -> bool {
        self.title.is_some() && self.link.is_some() && self.description.is_some()
    }
    pub fn build(self) -> Result<Item, Box<dyn std::error::Error>>{
        if self.ready() {
            return Ok(Item { title:self.title.unwrap() , link:self.link.unwrap(), description:self.description.unwrap()})
        }
        Err(Box::new(MyError::from_str("title, link, and description required before building.")))
    }
}