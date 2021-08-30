import path from 'path'
import dotenv from 'dotenv'

export function config(name = 'local'): dotenv.DotenvConfigOutput {
    return dotenv.config({
        path: path.join(path.dirname(__dirname), `${name}.env`),
    })
}
