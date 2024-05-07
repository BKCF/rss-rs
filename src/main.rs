use std::fs::File;
use std::io::{self,BufRead};
use roxmltree::Node;

use rss_rs::channel::{Channel, ChannelBuilder, RSSVersion};
use rss_rs::item::{Item, ItemBuilder};
use rss_rs::my_error::MyError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let urls = read_urls_from_file(&"urls.txt".to_string()).unwrap();
    let mut channels:Vec<Channel> = Vec::new();

    let mut iter = urls.iter();
    while let Some(url) = iter.next(){
        println!("{url:#?}");
        match get_text(&url).await {
            Ok(content) => channels.push(parse_xml_to_channel(&content)?),
            Err(error) => println!("read error for \'{url}\': {error}"),
        }
    }
    Ok(())
}

fn parse_xml_to_channel(content: &str) -> Result<Channel, Box<dyn std::error::Error>>{
    let doc = roxmltree::Document::parse(content)?;// {

    let mut channel_builder = ChannelBuilder::new();

    let rss_tag = match doc.descendants().find(|n| n.has_tag_name("rss")) {
        Some(elem) => elem,
        None => return Err(Box::new(MyError::from_str("No RSS tag.")))
    };

    //todo less jank
    match rss_tag.attribute("version") {
        Some(version) => {
            let version_split:Vec<&str> = version.split(".").collect();
            if version_split.len() >= 2 {
                channel_builder.set_version(&RSSVersion {major:version_split[0].parse()?, minor: version_split[1].parse()?});
            }
        }
        None => return Err(Box::new(MyError::from_str("No rss version number provided")))
    }

    let channel_tag = match rss_tag.descendants().find(|n| n.has_tag_name("channel")) {
        Some(elem) => elem,
        None => return Err(Box::new(MyError::from_str("No channel tag")))
    };

    let mut iterator = channel_tag.descendants();
    while let Some(node) = iterator.next() {
        if !node.is_element() {continue}
        
        if let Some(text) = node.text(){
            let tag_name = node.tag_name().name();

            if tag_name == "title" {
                channel_builder.set_title(&String::from(text))
            }
            else if tag_name == "link" {
                channel_builder.set_link(&String::from(text))
            }
            else if tag_name == "description" {
                channel_builder.set_description(&String::from(text))
            }
            else if tag_name == "item" {
                channel_builder.add_item(parse_item(&node)?);
            }
        }
    }

    Ok(channel_builder.build()?)


}

fn parse_item(item_tag: &Node) -> Result<Item, Box<dyn std::error::Error>> {
    let mut item_builder = ItemBuilder::new();

    let mut iterator = item_tag.descendants();
    while let Some(node) = iterator.next() {
        if !node.is_element() {continue}
        
        if let Some(text) = node.text(){
            let tag_name = node.tag_name().name();

            if tag_name == "title" {
                item_builder.set_title(&String::from(text));
            }
            else if tag_name == "link" {
                item_builder.set_link(&String::from(text));
            }
            else if tag_name == "description" {
                item_builder.set_description(&String::from(text));
            }
        }
    }
    item_builder.build()
}

async fn get_text(url: &String) -> Result<String, reqwest::Error>{
    reqwest::get::<&String>(&url).await?.text().await
}

fn read_urls_from_file(filepath: &String) -> std::io::Result<Vec<String>>{
    let file = File::open(filepath)?;
    let reader = io::BufReader::new(file);

    let urls: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .collect();
    Ok(urls)
}