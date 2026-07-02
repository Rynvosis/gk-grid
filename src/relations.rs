use bevy::prelude::*;

#[derive(Component)]
#[relationship(relationship_target = Tilemaps)]
pub struct TilemapOf(pub Entity); //on the tilemap, points at its grid

#[derive(Component)]
#[relationship_target(relationship = TilemapOf)]
pub struct Tilemaps(Vec<Entity>);
