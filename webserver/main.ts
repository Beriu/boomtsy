import express from "express";
import { Server } from "socket.io";
import http from "http";
import path from "path";
import bodyParser from "body-parser";
import { authenticate, fetchUser } from "./services/Discord";
import isAuthenticated from "./middlewares/isAuthenticated";
import JWT from "jsonwebtoken";
import createContex from "./middlewares/createContex";
import { Collection } from "discord.js";
import Session from "../bot/src/services/Session";

type DiscordUser = { id: string, username: string, avatar: string };

function generateAccessToken(userDiscordToken: string, expiresIn: number, user: DiscordUser) {
    return JWT.sign(
        { token: userDiscordToken, user },
        process.env.DISCORD_CLIENT_SECRET as string,
        { expiresIn: `${expiresIn} s` }
    );
}

export default async function runServer(sessions: Collection<string, Session>) {
   
    const app = express();
    const server = http.createServer(app);
    const socket = new Server(server);

    app.use(createContex());
    app.use(bodyParser.urlencoded({ extended: false }));
    app.use(bodyParser.json());
    
    app.use('', express.static(path.join(process.env.PWD as string, '/distApp')));

    app.get('/', function(req, res) {
        res.sendFile(path.join(process.env.PWD as string, '/distApp/index.html'));
    });

    app.get('/api/user', isAuthenticated, async (req, res) => {
        const user = await fetchUser(req.context!.discordToken!);
        res.json(user);
    });

    app.get('/api/user/connections', isAuthenticated, async (req, res) => {
        const session = sessions.find(sesh => sesh.hasUser(req.context!.user.id));
        if(session) return res.json(session.songs());
        res.status(400).send({ error: "Not active sessions." });
    });

    app.post('/api/auth', async (req, res) => {
        try {
            const { code, redirectUri } = req.body;
            if(!code) { res.status(401).send('Missing code!'); }

            const { access_token: accessToken, expires_in: expiresIn } = await authenticate(code, redirectUri);
            const user = await fetchUser(accessToken);
            const jwtToken = generateAccessToken(accessToken, expiresIn, user);
            res.send({ accessToken: jwtToken, expiresIn });
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