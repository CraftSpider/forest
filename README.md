
# Forest

A holding place for tree implementations

## Ownership Analysis

### object_tree::Tree

- `Tree` is owned by some item.
- The tree has a RefCell to the inner tree containing the actual data
- Borrowing an item from the tree immutably borrows the tree so that the item can reference back to it
- Items are inside a NonNull<RefCell<T>>
  - The NonNull allows the Ref to live as long as the borrow of Tree
    - This is sound as the Ref to the inner tree is no longer in use after dereferencing the pointer
    - This is sound as the NonNull will not be invalidated without either:
    - A unique mutable reference, which will be consumed
    - The tree going out of scope, which requires no borrows to it live
  - The RefCell tracks whether an item is currently borrowed
