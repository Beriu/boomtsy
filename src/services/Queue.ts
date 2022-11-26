export default class Queue<T> {

    constructor(private items: T[] = []) {}
    
    enqueue(element: T) {
        return this.items.push(element);
    }

    dequeue() {
        if(this.items.length > 0) {
            return this.items.shift();
        }
        throw Error("Atempting to deque without on zero items.");
    }
    
    peek() {
        return this.items.at(-1);
    }
    
    isEmpty(){
       return this.items.length === 0;
    }
   
    size(){
        return this.items.length;
    }
 
    clear(){
        this.items = [];
    }

    getItems() {
        return [...this.items];
    }
}
