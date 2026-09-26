use mundareu_world::Block;

pub const WINDOW_TITLE: &str = "Mundaréu";
pub const JUMP: &str = "Pular";
pub const PLACE: &str = "Colocar";
pub const BREAK: &str = "Quebrar";
pub const FRAMES_PER_SECOND: &str = "Quadros por segundo";

pub const fn block_name(block: Block) -> &'static str {
    match block {
        Block::Air => "Ar",
        Block::Grass => "Grama",
        Block::Dirt => "Terra",
        Block::Stone => "Pedra",
        Block::Sand => "Areia",
        Block::Wood => "Madeira",
        Block::Leaves => "Folhas",
        Block::Brick => "Tijolo",
    }
}
