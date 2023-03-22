use std::ops::Index;

pub fn find_largest_subset(
    a: &(impl Index<usize, Output = u8> + ?Sized),
    a_size: usize,
    b: &(impl Index<usize, Output = u8> + ?Sized),
    b_size: usize,
) -> (usize, usize) {
    let (mut final_start, mut final_size): (usize, usize) = (0, 0);

    let mut start: usize = 0;
    while start < a_size - final_size && final_size < b_size {
        let mut size: usize = 0;
        while size < b_size && a[start + size] == b[size] {
            size += 1;
        }

        if size > final_size {
            (final_start, final_size) = (start, size);
        }

        start += 1;
    }

    (final_start, final_size)
}

mod tests {
    use crate::utility::find_largest_subset;

    #[test]
    fn largest_subset_ok_test() {
        let a: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let b: &[u8] = &[7, 8, 9, 10];
        let res = find_largest_subset(a, a.len(), b, b.len());
        assert_eq!(res, (15usize, 4));
    }

    #[test]
    fn largest_subset_no_common_test() {
        let a: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let b: &[u8] = &[12, 13, 14, 15];
        let res = find_largest_subset(a, a.len(), b, b.len());
        assert_eq!(res, (0, 0));
    }
}
