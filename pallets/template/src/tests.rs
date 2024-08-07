use crate::{mock::*, Error, Event, Something, StorageType};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(TemplateModule::do_something(
			RuntimeOrigin::signed(1),
			StorageType { a: 1, b: 2 }
		));
		// Read pallet storage and assert an expected result.
		assert_eq!(Something::<Test>::get(), Some(StorageType { a: 1, b: 2 }));
		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::SomethingStored { something: StorageType { a: 1, b: 2 }, who: 1 }.into(),
		);
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(
			TemplateModule::cause_error(RuntimeOrigin::signed(1)),
			Error::<Test>::NoneValue
		);
	});
}
