from fastapi import FastAPI
from pydantic import BaseModel

app = FastAPI()

class Item(BaseModel):
    name: str
    price: float
    is_offer: bool = None

@app.get("/items")
def read_items():
    return []


@app.post("/items")
def create_item(item: Item):
    return item
