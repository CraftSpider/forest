
# Forest

A holding place for tree implementations

## Tree Implementations

### simple_tree::Tree

Pros:
- Is easily `Send`/`Sync`
- No extra allocation costs
Cons:
- Can only borrow nodes mutably *or* immutably
- Can't traverse the tree from a node borrow

### object_tree::Tree

Pros:
- Can have many nodes borrowed with different mutability at once
- Can work primarily through borrowed nodes
Cons:
- Not `Send`/`Sync` without the `atomic` feature
- Have to pay extra allocation costs

