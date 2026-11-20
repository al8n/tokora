//! A segment as a storage, alone: its own `K` frames, and the frame below read and written through
//! the link once its own are gone. The machine over chains of segments is driven in
//! `pratt/resolve/tests.rs`. Each cell names `core` alone, so the file runs in every tier.

use super::*;
use hybrid_arraydeque::typenum::{U2, U4};

#[test]
fn a_segment_takes_its_own_k_frames_and_reads_the_frame_below_once_they_are_gone() {
  let mut below: ArrayDeque<u8, U4> = ArrayDeque::new();
  for item in 0..4 {
    assert_eq!(below.push_back(item), None);
  }

  let mut upper: Segment<'_, u8, U2> = Segment::over(&mut below);
  assert_eq!(upper.len(), 0);
  assert_eq!(upper.max_capacity(), 2);
  assert!(upper.drained());
  // With none of its own, `last` and `last_mut` are the frame below, and `pop` never reaches it.
  assert_eq!(upper.last(), Some(&3));
  *upper.last_mut().unwrap() = 30;
  assert_eq!(upper.last(), Some(&30));
  assert_eq!(upper.pop(), None);
  assert_eq!(upper.first(), None);

  assert_eq!(upper.push(4), Ok(()));
  assert!(!upper.drained());
  assert_eq!(upper.push(5), Ok(()));
  // Its own two are in use: the refusal hands the item back, and takes nothing from below.
  assert_eq!(upper.push(6), Err(6));
  assert_eq!(upper.len(), 2);
  assert_eq!(upper.first(), Some(&4));
  assert_eq!(upper.last(), Some(&5));
  *upper.last_mut().unwrap() = 50;
  assert_eq!(upper.pop(), Some(50));
  assert_eq!(upper.pop(), Some(4));
  assert!(upper.drained());
  assert_eq!(upper.last(), Some(&30));
  drop(upper);

  // The storage below kept its four, and the one write that crossed the link.
  assert_eq!(below.len(), 4);
  assert_eq!(below.back(), Some(&30));
  assert_eq!(below.front(), Some(&0));
}

#[test]
fn the_first_segment_has_nothing_below_it_and_is_never_drained() {
  let mut root: Segment<'_, u8, U2> = Segment::root();
  assert!(!root.drained());
  assert_eq!(root.last(), None);
  assert!(root.last_mut().is_none());
  assert_eq!(root.pop(), None);
  assert_eq!(root.push(1), Ok(()));
  assert_eq!(root.push(2), Ok(()));
  assert_eq!(root.push(3), Err(3));
  assert_eq!(root.pop(), Some(2));
  assert_eq!(root.pop(), Some(1));
  assert!(!root.drained());
}

/// A segment over a segment: the link reaches the top of the one directly below, and only that.
#[test]
fn a_segment_over_a_segment_reads_the_top_of_the_one_below() {
  let mut root: Segment<'_, u8, U2> = Segment::root();
  assert_eq!(root.push(1), Ok(()));
  assert_eq!(root.push(2), Ok(()));
  {
    let mut middle: Segment<'_, u8, U2> = Segment::over(&mut root);
    assert_eq!(middle.push(3), Ok(()));
    assert_eq!(middle.push(4), Ok(()));
    {
      let mut upper: Segment<'_, u8, U2> = Segment::over(&mut middle);
      assert_eq!(upper.last(), Some(&4));
      *upper.last_mut().unwrap() = 40;
    }
    assert_eq!(middle.pop(), Some(40));
    assert_eq!(middle.pop(), Some(3));
    assert!(middle.drained());
    assert_eq!(middle.last(), Some(&2));
  }
  assert_eq!(root.pop(), Some(2));
  assert_eq!(root.pop(), Some(1));
}
