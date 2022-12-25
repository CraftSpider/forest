
# Forest

A holding place for tree implementations

## Tree Implementations

### simple_tree::Tree

Pros:
- Is easily `Send`/`Sync`
- No extra allocation or synchronization costs

Cons:
- Can only borrow nodes mutably *or* immutably
- Can only traverse the tree from certain borrows

### object_tree::Tree

Pros:
- Can have many nodes borrowed with different mutability at once
- Can work primarily through borrowed nodes

Cons:
- Not `Send`/`Sync` without the `atomic` feature
- Have to pay extra allocation and synchronization costs

