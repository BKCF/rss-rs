use std::fs::File;
use std::io::{self,BufRead};
use std::iter::Product;
use egui::{Align2, Color32, Context, Label, Sense, Window};
use roxmltree::Node;

use rss_rs::channel::{self, Channel, ChannelBuilder, RSSVersion};
use rss_rs::item::{Item, ItemBuilder};
use rss_rs::my_error::MyError;

use eframe::egui;
use tokio::task;
use poll_promise::Promise;

#[derive(Debug)]
pub struct Source {
    channel: Option<Channel>,
    url: String,
}
async fn read_new_urls(sources:&mut Vec<Source>){
    // let mut iter = sources.iter();
    // while let Some(mut source) = iter.next(){
    for source in sources{

        println!("{source:#?}");
        if source.channel.is_none() {
            match get_text(&source.url).await {
                Ok(content) => {
                    match parse_xml_to_channel(&content){
                        Ok(channel) => {
                         //   println!("{:?}", channel);
                            source.channel = Some(channel);
                        },
                        Err(error) => {
                            println!("{:?}", error);
                        },
                    }
                },
                Err(error) => println!("read error for \'{:?}\': {error}", source.url),
            }
        }
        
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut sources:Vec<Source> = Vec::new();

    for url in read_urls_from_file(&"urls.txt".to_string())? {
        sources.push(Source { channel: None, url: url });    
    }

    start_app(&mut sources)
}

fn start_app<'a>(sources:&'a mut Vec<Source>) -> Result<(), Box<dyn std::error::Error>>{
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0,400.0]),
        ..Default::default()
    }; 
   // let promise:poll_promise::Promise<String> = poll_promise::Promise::new();//<String>;
    let mut open:bool = false;
    Ok(eframe::run_simple_native("RSS Reader Deluxe(tm)", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {

            let promise = Promise::spawn_thread("slow_operation", move || read_new_urls());
            if let Some(String) = promise.ready() {
                
            }
            

            ui.horizontal(|ui| {
                ui.collapsing("feeds", |ui| {
                    if ui.button("add new").clicked(){
                        open = true;
                    }
                    for source in sources.iter() {
                        if let Some(channel) = source.channel.as_ref() {
                            ui.label(&channel.title);
                        }
                    }
                    if open {
                        new_feed_window(&ctx, &mut open, &mut sources);
                    }
                    
                });
            });
            egui::ScrollArea::vertical().show(ui,|ui| {
                for source in sources.iter() {
                    if let Some(channel) = source.channel.as_ref() {
                        for item in channel.items.iter() {
                            ui.horizontal(|ui|{
                                ui.label(egui::RichText::new(&channel.title).color(Color32::DARK_GRAY).background_color(Color32::LIGHT_GRAY));
                                ui.label(egui::RichText::new("::"));

                                let title_label = egui::Label::new(&item.title).selectable(false).sense(Sense::hover());
                                ui.add(title_label);
                            });
                        }
                    }
                }
            });
        });
    })?)
    //DO NOT TOUCH
}



fn new_feed_window(ctx:&Context, open:&mut bool, sources:&mut Vec<Source>) -> Option<egui::InnerResponse<Option<()>>>{
    let mut text = "".to_owned();

    egui::Window::new("My Window").open(open).default_pos([85.0,20.0]).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("url");
            
            ui.text_edit_singleline(&mut text);    
            if ui.button("add").clicked() {
                sources.push(Source{channel:None,url:text});
            }        
        });
     })
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

    let mut iterator = channel_tag.children();
    while let Some(node) = iterator.next() {
     //   println!("{:?}", node.tag_name());
        match node.node_type(){
            roxmltree::NodeType::Element => {
                let tag_name = node.tag_name().name();

                if tag_name == "item" {
                    channel_builder.add_item(parse_item(&node)?);
                }
                else if let Some(text) = node.text(){
                    if tag_name == "title" {
                        channel_builder.set_title(&String::from(text))
                    }
                    else if tag_name == "link" {
                        channel_builder.set_link(&String::from(text))
                    }
                    else if tag_name == "description" {
                        channel_builder.set_description(&String::from(text))
                    }
                }
            }
            _ => ()
        }
    }
    Ok(channel_builder.build()?)
}

fn parse_item(item_tag: &Node) -> Result<Item, Box<dyn std::error::Error>> {
    let mut item_builder = ItemBuilder::new();

    let mut iterator = item_tag.children();
    while let Some(node) = iterator.next() {
        
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