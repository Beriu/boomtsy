import {Interaction} from "discord.js";

type ValidatorFunction = <T>(value: T) => boolean | unknown;

type ValidationMap = { [key: string]: ValidatorFunction };

const validations: ValidationMap  = {
    'isGuild': (interaction) => true
};

export default function createValidators(validators: Array<keyof ValidationMap>) {

}