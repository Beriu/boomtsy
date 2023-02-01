import webserver from "./webserver/main";
import bot from "./bot/src/main";
import { Collection } from "discord.js";
import Session from "./bot/src/services/Session";
import EventEmitter from "node:events";

const sessions: Collection<string, Session> = new Collection(); 
const bridge = new EventEmitter();

void webserver(sessions, bridge);

void bot(sessions, bridge);
