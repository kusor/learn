fn main() {
    let answer: Option<f32> = median(vec![1.0, 2.0, 5.0]);
    println!("median[1,2,5] = {:?}", answer);
}

fn median(list: Vec<f32>) -> Option<f32> {
    if list.is_empty() {
        return None
    }
    let mut sorted = list.clone();
    sorted.sort_by(f32::total_cmp);
    let pos = sorted.len();
    let mid = pos / 2;
    if pos % 2 != 0 {
        return Some(sorted[mid])
    }
    Some((sorted[mid] + sorted[mid-1])/2.0)
}

#[test]
fn empty_list() {
    assert_eq!(median(vec![]), None);
}

#[test]
fn sorted_list() {
    let list = vec![1.0,4.0,5.0];
    assert_eq!(median(list), Some(4.0));
}

#[test]
fn unsorted_list() {
    let another_list = vec![3.0,1.5,8.8,5.0];
    assert_eq!(median(another_list), Some(4.0));
}
