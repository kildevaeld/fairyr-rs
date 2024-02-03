import { TEST } from "./other.js";
import { GREETING as Greeting } from "./shared.js";
import List from "./list.js";

import { Todo } from "./models.js";

const list = new List();

list.add("Hello, World");
// list.addTask(new Task());
console.log(`${Greeting}, ${TEST}`);
