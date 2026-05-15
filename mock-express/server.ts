import express, { Request, Response } from 'express';

const app = express();

interface CreateUserDto {
    username: string;
    email: string;
    age: number;
    tags: string[];
}

app.post('/users', (req: Request<{}, {}, CreateUserDto>, res: Response) => {
    res.json(req.body);
});

app.put('/users/:id', (req: Request<{id: string}, {}, CreateUserDto>, res: Response) => {
    res.json(req.body);
});
