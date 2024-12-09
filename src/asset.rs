use bevy::prelude::*;
use bevy::asset::*;
use crate::types::*;
pub fn get_guard_sprite(direction:&Direction,index:usize,asset_server: Res<AssetServer>) -> Sprite {
    match direction {
        Direction::Up => match index {
            2 => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Up2.png")),
            3 => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Up3.png")),
            _ => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Up1.png")),
        },
        Direction::Down => match index {
            2 => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Down2.png")),
            3 => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Down3.png")),
            _ => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Down1.png")),
        },
        Direction::Left => match index {
            2 => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Left2.png")),
            3 => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Left3.png")),
            _ => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Left1.png")),
        },
        Direction::Right => match index {
            2 => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Right2.png")),
            3 => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Right3.png")),
            _ => Sprite::from_image(asset_server.load("embedded://day6vis/sprites/Right1.png")),
        },
    }
}

pub struct EmbeddedPlug;
impl Plugin for EmbeddedPlug {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "sprites/Up1.png");
        embedded_asset!(app, "sprites/Up2.png");
        embedded_asset!(app, "sprites/Up3.png");
        embedded_asset!(app, "sprites/Right1.png");
        embedded_asset!(app, "sprites/Right2.png");
        embedded_asset!(app, "sprites/Right3.png");
        embedded_asset!(app, "sprites/Down1.png");
        embedded_asset!(app, "sprites/Down2.png");
        embedded_asset!(app, "sprites/Down3.png");
        embedded_asset!(app, "sprites/Left1.png");
        embedded_asset!(app, "sprites/Left2.png");
        embedded_asset!(app, "sprites/Left3.png");
        embedded_asset!(app, "fonts/FiraSans-Bold.ttf");
    }
}
