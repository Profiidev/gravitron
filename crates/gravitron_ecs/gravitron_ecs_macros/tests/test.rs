#[cfg(test)]
mod test {
  use gravitron_ecs_macros::all_tuples;

  #[test]
  fn all_tuples() {
    trait Trait {
      fn call(self) -> Vec<String>;
    }

    trait Trait2 {
      fn into() -> String;
    }

    impl Trait2 for usize {
      fn into() -> String {
        "usize".into()
      }
    }

    impl Trait2 for isize {
      fn into() -> String {
        "isize".into()
      }
    }

    macro_rules! test_macro {
      ($($p:ident),*) => {
        impl<$($p : Trait2),*> Trait for ($($p ,)*) {
          fn call(self) -> Vec<String> {
            vec![$($p::into()),*]
          }
        }
      };
    }

    all_tuples!(test_macro, 1, 2, F);

    let t = (0usize, 1isize);
    let ret = t.call();
    assert_eq!(ret, vec!["usize", "isize"]);
  }
}
