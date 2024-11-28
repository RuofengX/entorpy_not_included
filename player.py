from dataclasses import dataclass
from typing import Dict, NamedTuple, Self


class Position(NamedTuple):
    x: float
    y: float


@dataclass
class Player:
    id: int
    name: str
    pos: Position
    health: float = 100.0
    speed: float = 10.0
    energy: float = 4000.0
    breath: float = 100


@dataclass
class Cell:
    pos: Position
    ty: str
    amount: float


@dataclass
class World:
    cells: Dict[Position, Cell]
    players: Dict[int, Player]