use crate::{
    mock::*, AmountOp, AssetRate, Error, MarketRate, RateAccount, RateAction,
};
use frame_support::{assert_noop, assert_ok};

fn last_event() -> Event {
    frame_system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

fn simple_market_rate() -> MarketRate<
    <Test as frame_system::Config>::AccountId,
    <Test as sugarfunge_asset::Config>::ClassId,
    <Test as sugarfunge_asset::Config>::AssetId,
> {
    MarketRate {
        rates: vec![
            //
            // Buyer wants these goods
            //
            // Market will transfer 1 asset of class_id: 2000 asset_id: 1 to buyer
            AssetRate {
                class_id: 2000,
                asset_id: 1,
                action: RateAction::Transfer,
                amount: 1,
                from: RateAccount::Market,
                to: RateAccount::Buyer,
            },
            // Market will mint 100 assets of class_id: 2000 asset_id: 2 for buyer
            AssetRate {
                class_id: 2000,
                asset_id: 2,
                action: RateAction::Mint,
                amount: 1,
                from: RateAccount::Market,
                to: RateAccount::Buyer,
            },
            //
            // Market asking price
            //
            // Market requires buyer owns 5000 or more assets of type class_id: 3000 asset_id: 0
            AssetRate {
                class_id: 3000,
                asset_id: 1,
                action: RateAction::Has(AmountOp::GreaterEqualThan),
                amount: 5000,
                from: RateAccount::Buyer,
                to: RateAccount::Market,
            },
            // Buyer will transfer 5 assets of type class_id: 3000 asset_id: 2 to market
            AssetRate {
                class_id: 3000,
                asset_id: 2,
                action: RateAction::Transfer,
                amount: 5,
                from: RateAccount::Buyer,
                to: RateAccount::Market,
            },
            // Market will burn 50 assets of class_id: 3000 asset_id: 3
            AssetRate {
                class_id: 3000,
                asset_id: 3,
                action: RateAction::Burn,
                amount: 50,
                from: RateAccount::Market,
                to: RateAccount::Market,
            },
            //
            // Royalties
            //
            // Market pays royalties to account 0
            AssetRate {
                class_id: 4000,
                asset_id: 1,
                action: RateAction::Transfer,
                amount: 2,
                from: RateAccount::Market,
                to: RateAccount::Account(0),
            },
            // Buyer pays royalties to account 0
            AssetRate {
                class_id: 4000,
                asset_id: 1,
                action: RateAction::Transfer,
                amount: 1,
                from: RateAccount::Buyer,
                to: RateAccount::Account(0),
            },
        ],

        metadata: vec![],
    }
}

pub fn before_market() {
    run_to_block(10);

    assert_ok!(Asset::do_create_class(&1, &1, 2000, [0].to_vec()));
    assert_ok!(Asset::do_create_class(&1, &1, 3000, [0].to_vec()));
    assert_ok!(Asset::do_create_class(&1, &1, 4000, [0].to_vec()));

    let asset_ids = [1, 2, 3, 4, 5].to_vec();
    let amounts = [100, 200, 300, 400, 500].to_vec();

    assert_ok!(Asset::do_batch_mint(
        &1,
        &2,
        2000,
        asset_ids.clone(),
        amounts.clone(),
    ));

    assert_ok!(Asset::do_batch_mint(
        &1,
        &2,
        3000,
        asset_ids.clone(),
        amounts.clone(),
    ));

    assert_ok!(Asset::do_batch_mint(
        &1,
        &2,
        4000,
        asset_ids.clone(),
        amounts.clone(),
    ));

    assert_ok!(Market::do_create_market(&1, 1000));

    assert_eq!(
        last_event(),
        Event::Market(crate::Event::Created {
            market_id: 1000,
            who: 1,
        }),
    );
}

#[test]
fn before_market_works() {
    new_test_ext().execute_with(|| {
        before_market();
    })
}

#[test]
fn create_market_works() {
    new_test_ext().execute_with(|| {
        before_market();

        assert_ok!(Market::do_create_market(&1, 1001));

        assert_eq!(
            last_event(),
            Event::Market(crate::Event::Created {
                market_id: 1001,
                who: 1,
            }),
        );
    })
}

#[test]
fn create_market_rate_works() {
    new_test_ext().execute_with(|| {
        before_market();

        let market_rate = simple_market_rate();

        assert_ok!(Market::do_create_market_rate(&1, 1000, 2, &market_rate));
    })
}

#[test]
fn invalid_market_fails() {
    new_test_ext().execute_with(|| {
        before_market();

        assert_noop!(
            Market::do_compute_deposit(&1, 1001, 1, 2),
            Error::<Test>::InvalidMarket
        );
    })
}

#[test]
fn invalid_market_rate_fails() {
    new_test_ext().execute_with(|| {
        before_market();

        assert_noop!(
            Market::do_compute_deposit(&1, 1000, 1000, 2),
            Error::<Test>::InvalidMarketRate
        );
    })
}

#[test]
fn compute_deposit_fails() {
    new_test_ext().execute_with(|| {
        before_market();

        assert_ok!(Market::do_create_market(&2, 2000));

        let market_rate = simple_market_rate();

        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &market_rate));

        let result = Market::do_compute_deposit(&2, 2000, 100, 10);
        if let Ok((can_deposit, deposits)) = result {
            assert_eq!(can_deposit, false);
            assert_eq!(deposits.get(&market_rate.rates[0]), Some(&10));
            assert_eq!(deposits.get(&market_rate.rates[1]), None);
            assert_eq!(deposits.get(&market_rate.rates[2]), None);
            assert_eq!(deposits.get(&market_rate.rates[3]), None);
            assert_eq!(deposits.get(&market_rate.rates[4]), Some(&-200));
            assert_eq!(deposits.get(&market_rate.rates[5]), Some(&20));
            assert_eq!(deposits.get(&market_rate.rates[6]), None);
        } else {
            result.unwrap();
        };
    })
}

#[test]
fn compute_deposit_works() {
    new_test_ext().execute_with(|| {
        before_market();

        assert_ok!(Market::do_create_market(&2, 2000));

        let market_rate = simple_market_rate();

        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &market_rate));

        let result = Market::do_compute_deposit(&2, 2000, 100, 2);
        if let Ok((can_deposit, deposits)) = result {
            assert_eq!(can_deposit, true);
            assert_eq!(deposits.get(&market_rate.rates[0]), Some(&2));
            assert_eq!(deposits.get(&market_rate.rates[1]), None);
            assert_eq!(deposits.get(&market_rate.rates[2]), None);
            assert_eq!(deposits.get(&market_rate.rates[3]), None);
            assert_eq!(deposits.get(&market_rate.rates[4]), Some(&100));
            assert_eq!(deposits.get(&market_rate.rates[5]), Some(&4));
            assert_eq!(deposits.get(&market_rate.rates[6]), None);

            // assert_eq!(deposits, DepositBalances::default());
        } else {
            result.unwrap();
        };
    })
}

#[test]
fn deposit_assets_works() {
    new_test_ext().execute_with(|| {
        before_market();
        assert_ok!(Market::do_create_market(&2, 2000));
        let market_rate = simple_market_rate();
        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &market_rate));
        assert_ok!(Market::do_deposit_assets(&2, 2000, 100, 2));
    })
}

#[test]
fn deposit_assets_fails() {
    new_test_ext().execute_with(|| {
        before_market();
        assert_ok!(Market::do_create_market(&2, 2000));
        let market_rate = simple_market_rate();
        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &market_rate));
        assert_noop!(
            Market::do_deposit_assets(&2, 2000, 100, 10),
            Error::<Test>::InsufficientAmount
        );
    })
}
