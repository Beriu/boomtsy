import {slashCommand} from "../types";
import {AxiosInstance} from "axios";

export default class SlashCommandRepository {

    constructor(
        protected axios: AxiosInstance
    ) {}

    async read(): Promise<slashCommand[]> {
        const {data} = await this.axios.get('/commands');
        return data;
    }

    async create(cmd: slashCommand): Promise<slashCommand> {
        const {data} = await this.axios.post('/commands', cmd);
        return data;
    }

    async delete(id: string) {
        const {data} = await this.axios.delete(`/commands/${id}`);
        return data;
    }
}