import express from "express";
import { Server } from "socket.io";
import http from "http";
import path from "path";
import bodyParser from "body-parser";
import { authenticate, fetchUser } from "./services/Discord";
import isAuthenticated from "./middlewares/isAuthenticated";
import JWT from "jsonwebtoken";
import createSession from "./middlewares/createSession";

function generateAccessToken(userDiscordToken: string, expiresIn: number) {
    return JWT.sign(
        { token: userDiscordToken },
        process.env.DISCORD_CLIENT_SECRET as string,
        { expiresIn: `${expiresIn} s` }
    );
}

export default async function runServer() {
   
    const app = express();
    const server = http.createServer(app);
    const socket = new Server(server);
    const sessions = new Map();

    app.use(createSession());
    app.use(bodyParser.urlencoded({ extended: false }));
    app.use(bodyParser.json());
    
    app.use('', express.static(path.join(process.env.PWD as string, '/distApp')));

    app.get('/', function(req, res) {
        res.sendFile(path.join(process.env.PWD as string, '/distApp/index.html'));
    });

    app.get('/api/user', isAuthenticated, async (req, res) => {
        const user = await fetchUser(req.session.discordToken!);
        res.send(user);
    });

    app.post('/api/auth', async (req, res) => {
        try {
            const { code, redirectUri } = req.body;
            if(!code) { res.status(401).send('Missing code!'); }

            const response = await authenticate(code, redirectUri);
            const jwtToken = generateAccessToken(response.access_token, response.expires_in);
            sessions.set(
                jwtToken,
                response
            );
            res.send({ accessToken: jwtToken, expiresIn: response.expires_in });
        } catch(error: any) {
            res.status(401).send(`Error: ${error.message ?? 'no msg.'}`);
        }
    });

    socket.on('/api/ws', (socket) => {
        socket.emit("sessions/get", 'Hi there! This is on CibzPi');
    });

    server.listen(process.env.WEBSERVER_PORT, () => {
        console.log(`Webserver running at http://localhost:${process.env.WEBSERVER_PORT}`);
    });
}