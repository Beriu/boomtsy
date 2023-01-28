import webserver from "./webserver/main";
import bot from "./bot/src/main";
import { Collection } from "discord.js";
import Session from "./bot/src/services/Session";

const sessions: Collection<string, Session> = new Collection(); 

void webserver(sessions);

void bot(sessions);
