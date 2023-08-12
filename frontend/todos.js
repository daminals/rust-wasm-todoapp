const addTodoForm = document.getElementById('addTodoForm')
const todoInput = document.getElementById('todoInput')
const todoList = document.getElementById('todoList')

addTodoForm.addEventListener('submit', async (event) => {
  event.preventDefault()

  const todoText = todoInput.value
  if (!todoText) return

  // Update local state immediately
  const todo = todoText;
  console.log(todo)
  addTodoToLocalState(todo);

  const response = await fetch('api/todo/', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ text: todoText })
  })

  console.log(response)

  if (response.ok) {
    todoInput.value = ''
  }
})

async function fetchTodos() {
  const response = await fetch('api/todo/')
  const unpackTodosJson = await response.json()
  console.log(unpackTodosJson)
  const todos = unpackTodosJson["keys"]

  todoList.innerHTML = ''
  todos.forEach(todo => {
    const li = document.createElement('li')
    li.textContent = todo

    const deleteButton = document.createElement('button')
    deleteButton.textContent = 'Delete'
    deleteButton.classList.add('deleteButton')
    deleteButton.addEventListener('click', async () => {
      // Update local state to remove the deleted todo
      deleteTodoFromLocalState(todo);

      // update kv
      await fetch(`/api/todo/`, {
        method: 'DELETE',
        body: JSON.stringify({ text: todo })
      })
    })

    li.appendChild(deleteButton)
    todoList.appendChild(li)
  })
}

document.addEventListener('DOMContentLoaded', async () => {
  await fetchTodos();
});

// Update local state to add a new todo
function addTodoToLocalState(todo) {
  console.log(todo)
  const li = document.createElement('li');
  li.textContent = todo;

  const deleteButton = document.createElement('button');
  deleteButton.textContent = 'Delete';
  deleteButton.addEventListener('click', async () => {
    // Update local state to remove the deleted todo
    deleteTodoFromLocalState(todo);
    await fetch(`/api/todo/`, {
      method: 'DELETE',
      body: JSON.stringify({ text: todo })
    });
  });

  li.appendChild(deleteButton);
  todoList.appendChild(li);
}

// Update local state to delete a todo
function deleteTodoFromLocalState(todo) {
  const todoElements = todoList.querySelectorAll('li');
  for (const element of todoElements) {
    const searchText = todo + 'Delete';
    console.log(element.textContent, searchText, todo)
    if (element.textContent === searchText) {
      element.remove();
      break;
    }
  }
}

