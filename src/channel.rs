use std::collections::HashMap;
use crate::item::Item;

#[derive(Debug)]
#[derive(Copy)]
#[derive(Clone)]
pub struct RSSVersion{
    pub major:u8,
    pub minor:u8,
}

#[derive(Debug)]
pub struct Channel<'a>{
    //see: https://www.rssboard.org/rss-specification#requiredChannelElements
    pub version: RSSVersion,
    pub title: String,
    pub link: String,
    pub description: String,
    pub props: HashMap<String, String>,
    pub items: Vec<&'a Item>
}

#[derive(Debug)]
pub struct ChannelBuilder<'a>{
    //see: https://www.rssboard.org/rss-specification#requiredChannelElements
    version: Option<RSSVersion>,
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    props: HashMap<String, String>,
    items: Vec<&'a Item>,
}

impl<'a> ChannelBuilder<'a>{
    pub fn new() -> Self {
        Self{version:None, title:None, link:None, description:None, props:HashMap::new(), items:Vec::new()}
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
    pub fn set_version(&mut self, version: &RSSVersion){
        self.version = Some(version.clone());
    }
    pub fn add_item(&mut self, item: &'a Item){
        self.items.push(&item);
    }

   
    pub fn ready(&self) -> bool {
        match self.version {
            Some(version) =>{
                if !(version.major == 2 && version.minor == 0) {
                    false
                }else{
                    self.title.is_some() && self.link.is_some() && self.description.is_some()
                }
            }
            None => false
        }
    }

    pub fn build<'b>(self) -> Result<Channel<'a>, &'static str> {
        if self.ready() {
            return Ok(Channel { version:self.version.unwrap(), title:self.title.unwrap() , link:self.link.unwrap(), description:self.description.unwrap(), props:self.props, items:self.items})
        }
        Err("RSS V2, title, link, and description are required")
    }
}