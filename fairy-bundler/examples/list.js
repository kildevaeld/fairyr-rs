import { Todo } from "./models.js";
var count = 0;

export default class Todos {
  #todos = new Map();
  add(name) {
    let todo = new Todo(++count, name);
    this.#todos.set(todo.id, todo);
  }
}
