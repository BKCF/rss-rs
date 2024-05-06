use std::fs::File;
use std::io::{self,BufRead};
use std::collections::HashMap;
use reqwest::Error;
use tokio::sync::broadcast::error;
use std::borrow::Cow;

use rss_rs::channel::{Channel, ChannelBuilder, RSSVersion};
use rss_rs::item::Item;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    
    let urls = read_urls_from_file(&"urls.txt".to_string()).unwrap();

    for url in urls.iter() {
        println!("{url:#?}");
        match get_text(&url).await {
            Ok(content) => {
                match parse_xml_to_channel(&content) {
                    Ok(channel) => {
                        println!("{:?}",channel);
                        //todo we get a channel now
                        //feeds.insert(&channel.title, channel);
                    },
                    Err(error) => println!("xml parse error for \'{url}\': {error}"),
                }
                
            },
            Err(error) => println!("read error for \'{url}\': {error}")
        }

    }
    Ok(())
}

fn parse_xml_to_channel(content: &str) -> Result<Channel, &'static str>{
    let doc = match roxmltree::Document::parse(content) {
        Ok(result) => result,
        Err(_) => return Err("Parse error"),
    };

    let mut channel_builder = ChannelBuilder::new();

    let rss_tag = match doc.descendants().find(|n| n.has_tag_name("rss")) {
        Some(elem) => elem,
        None => return Err("no rss tag")
    };

    //todo make legit
    channel_builder.set_version(&RSSVersion { major: 2, minor: 0 });

    let channel_tag = match doc.descendants().find(|n| n.has_tag_name("channel")) {
        Some(elem) => elem,
        None => return Err("no channel tag")
    };
    let mut iterator = channel_tag.descendants();
    while let Some(node) = iterator.next() {
        if channel_tag.descendants().len() <= 0 {break} 

        let k = node.tag_name().name();
        println!("{k}");
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
                //TODO :3
            }
        }


    }


    Ok(channel_builder.build()?)


}


async fn get_text(url: &String) -> reqwest::Result<String>{
    match reqwest::get::<&String>(&url).await{
        Ok(resp) => resp.text().await, //future<Result>
        Err(error) => Err(error),      //Result::Err
    }
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