@id("ViewReservation")
permit(
  principal == ?principal,
  action in [Action::"viewReservation"],
  resource in ?resource);
  
@id("MemberReservation")
permit(
  principal == ?principal, 
  action in [Action::"viewReservation",
             Action::"updateReservation",
             Action::"createReservation"],
  resource in ?resource);

@id("AdminReservation")
permit(
  principal == ?principal,
  action in [Action::"viewReservation",
             Action::"updateReservation",
             Action::"createReservation",
             Action::"grantAccessReservation"],
  resource in ?resource);

@id("ViewPropertyOrHotel")
permit(
  principal == ?principal,
  action in [Action::"viewHotel",
             Action::"viewProperty"],
  resource in ?resource);
  
@id("MemberPropertyOrHotel")
permit(
  principal == ?principal, 
  action in [Action::"viewHotel",
             Action::"updateHotel",
             Action::"createHotel",
             Action::"viewProperty",
             Action::"updateProperty",
             Action::"createProperty"],
  resource in ?resource);

@id("AdminPropertyOrHotel")
permit(
  principal == ?principal,
  action in [Action::"viewHotel",
             Action::"updateHotel",
             Action::"createHotel",
             Action::"grantAccessHotel",
             Action::"viewProperty",
             Action::"updateProperty",
             Action::"createProperty",
             Action::"grantAccessProperty"],
  resource in ?resource);