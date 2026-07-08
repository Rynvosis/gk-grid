use bevy::prelude::*;

#[derive(Component, Debug)]
#[relationship(relationship_target = Tilemaps)]
pub struct TilemapOf(pub Entity); //on the tilemap, points at its grid

#[derive(Component, Debug)]
#[relationship_target(relationship = TilemapOf)]
pub struct Tilemaps(Vec<Entity>);
