use std::ops::Index;

pub fn find_largest_subset(
    a: &(impl Index<usize, Output = u8> + ?Sized),
    a_size: usize,
    b: &(impl Index<usize, Output = u8> + ?Sized),
    b_size: usize,
    extend_into_input: bool,
) -> (usize, usize) {
    let (mut final_start, mut final_size): (usize, usize) = (0, 0);
    let inner_max = if extend_into_input {
        a_size + b_size
    } else {
        a_size
    };

    let mut start_ptr: usize = 0;
    while start_ptr < a_size - final_size && final_size < b_size {
        let mut size: usize = 0;
        while size < b_size - 1
            && start_ptr + size < inner_max - 1
            && a[start_ptr + size] == b[size]
        {
            size += 1;
        }

        if size > final_size {
            (final_start, final_size) = (start_ptr, size);
        }

        start_ptr += 1;
    }

    (final_start, final_size)
}

#[cfg(test)]
mod tests {
    use crate::utility::find_largest_subset;

    #[test]
    fn largest_subset_ok_test() {
        let a: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let b: &[u8] = &[7, 8, 9, 10];
        let res = find_largest_subset(a, a.len(), b, b.len(), false);
        assert_eq!(res, (15usize, 4));
    }

    #[test]
    fn largest_subset_no_common_test() {
        let a: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let b: &[u8] = &[12, 13, 14, 15];
        let res = find_largest_subset(a, a.len(), b, b.len(), false);
        assert_eq!(res, (0, 0));
    }
}
